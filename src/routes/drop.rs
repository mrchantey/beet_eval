use super::*;
use crate::prelude::*;
use beet::prelude::*;

/// Request params for [`EvalDrop`], surfaced in `--help`.
#[derive(Reflect)]
struct DropParams {
	/// The package whose table holds the row.
	package: SmolStr,
	/// The table: `evals`, `rubrics`, `render` or `actions`.
	table: PackageTable,
	/// The row's key, its id, ie `market.competitors-named`.
	key: SmolStr,
}

/// `eval/drop`: removes one row of a package's table, refused while anything
/// still names it, an eval a rubric cites or a rubric a render spec builds.
/// The twin of `eval/put`: how a person or an agent retires a row, the
/// citations fixed first.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
#[require(
	PathPartial = PathPartial::new("drop"),
	ParamsPartial = ParamsPartial::new::<DropParams>()
)]
pub async fn EvalDrop(cx: ActionContext<Request>) -> Result<Response> {
	let params = cx.input.parse_params::<DropParams>()?;
	let workspace = LoadedWorkspace::of(&cx.caller).await?;
	let package = workspace.package(&params.package).ok_or_else(|| {
		bevyhow!("the workspace names no package {}", params.package)
	})?;
	let at = format!("{}/{}/{}", params.package, params.table, params.key);
	let tables = TableStore::new(package.store.clone());
	let key = params.key.as_str();
	let exists = match params.table {
		PackageTable::Evals => tables.table::<Eval>().exists(key).await?,
		PackageTable::Rubrics => tables.table::<Rubric>().exists(key).await?,
		PackageTable::Render => {
			tables.table::<RenderSpec>().exists(key).await?
		}
		PackageTable::Actions => {
			tables.table::<CoachAction>().exists(key).await?
		}
	};
	if !exists {
		return refusal(format!("{at} is no row\n")).xok();
	}
	let named_by = Drop::named_by(&workspace, package, params.table, key);
	if !named_by.is_empty() {
		return refusal(format!(
			"{at} is still named by {}\n",
			named_by.join(", ")
		))
		.xok();
	}
	match params.table {
		PackageTable::Evals => tables.table::<Eval>().remove(key).await?,
		PackageTable::Rubrics => tables.table::<Rubric>().remove(key).await?,
		PackageTable::Render => {
			tables.table::<RenderSpec>().remove(key).await?
		}
		PackageTable::Actions => {
			tables.table::<CoachAction>().remove(key).await?
		}
	}
	Response::ok_text(format!("dropped {at}\n")).xok()
}

/// What keeps a row from being dropped.
struct Drop;

impl Drop {
	/// Everything naming the row `key` of `table`: the rubrics citing an
	/// eval, the render specs building a rubric.
	fn named_by(
		workspace: &LoadedWorkspace,
		package: &LoadedPackage,
		table: PackageTable,
		key: &str,
	) -> Vec<String> {
		match table {
			PackageTable::Evals => workspace
				.rubrics()
				.iter()
				.filter(|(_, rubric)| {
					rubric
						.citations()
						.any(|citation| citation.eval.as_str() == key)
				})
				.map(|(reference, _)| reference.to_string())
				.collect(),
			PackageTable::Rubrics => package
				.render
				.iter()
				.filter(|spec| spec.rubric.as_str() == key)
				.map(|spec| {
					format!("{}/render/{}", package.manifest.name, spec.rubric)
				})
				.collect(),
			PackageTable::Render | PackageTable::Actions => Vec::new(),
		}
	}
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	#[beet::test]
	async fn drops_rows_nothing_names() {
		let mut fixture = Fixture::new().await;
		fixture
			.refused(
				"eval/drop --package=acme_biz --table=evals --key=product.origin",
			)
			.await
			.xpect_contains(
				"acme_biz/evals/product.origin is still named by acme_biz/owner",
			);
		fixture
			.refused(
				"eval/drop --package=acme_course --table=rubrics --key=01-business-plan",
			)
			.await
			.xpect_contains(
				"still named by acme_course/render/01-business-plan",
			);
		fixture
			.refused(
				"eval/drop --package=acme_biz --table=evals --key=product.nothing",
			)
			.await
			.xpect_contains("is no row");
		// a render spec names nothing, so it goes, and then its rubric may
		fixture
			.ok(
				"eval/drop --package=acme_course --table=render --key=01-business-plan",
			)
			.await
			.xpect_eq("dropped acme_course/render/01-business-plan\n");
		fixture
			.ok(
				"eval/drop --package=acme_course --table=rubrics --key=01-business-plan",
			)
			.await
			.xpect_eq("dropped acme_course/rubrics/01-business-plan\n");
		fixture.ok("eval/check").await.xpect_contains("0 error(s)");
	}
}
