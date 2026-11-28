use super::*;
use crate::prelude::*;
use beet::prelude::*;

/// `eval/drop`: removes one row of a suite's table, refused while anything
/// still names it, an eval a rubric cites or a rubric a render spec builds.
/// The twin of `eval/put`: how a person or an agent retires a row, the
/// citations fixed first.
#[action(route = "drop")]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
pub async fn EvalDrop(
	/// The suite whose table holds the row.
	#[param]
	suite: SmolStr,
	/// The table the row is removed from.
	#[param]
	table: SuiteTable,
	/// The row's key, its id, ie `market.competitors-named`.
	#[param]
	key: SmolStr,
	cx: ActionContext<Request>,
) -> Result<Response> {
	let workspace = LoadedWorkspace::of(&cx.caller).await?;
	let loaded = workspace
		.suite(&suite)
		.ok_or_else(|| bevyhow!("the workspace names no suite {}", suite))?;
	let at = format!("{}/{}/{}", suite, table, key);
	let store = &loaded.store;
	let key = key.as_str();
	let exists = match table {
		SuiteTable::Evals => {
			RowTable::<Eval>::new(store.clone()).exists(key).await?
		}
		SuiteTable::Rubrics => {
			RowTable::<Rubric>::new(store.clone()).exists(key).await?
		}
		SuiteTable::Render => {
			RowTable::<RenderSpec>::new(store.clone())
				.exists(key)
				.await?
		}
		SuiteTable::Actions => {
			RowTable::<CoachAction>::new(store.clone())
				.exists(key)
				.await?
		}
		SuiteTable::Claims => {
			RowTable::<Claim>::new(store.clone()).exists(key).await?
		}
	};
	if !exists {
		return refusal(format!("{at} is no row\n")).xok();
	}
	let named_by = Drop::named_by(&workspace, loaded, table, key);
	if !named_by.is_empty() {
		return refusal(format!(
			"{at} is still named by {}\n",
			named_by.join(", ")
		))
		.xok();
	}
	match table {
		SuiteTable::Evals => {
			RowTable::<Eval>::new(store.clone()).remove(key).await?
		}
		SuiteTable::Rubrics => {
			RowTable::<Rubric>::new(store.clone()).remove(key).await?
		}
		SuiteTable::Render => {
			RowTable::<RenderSpec>::new(store.clone())
				.remove(key)
				.await?
		}
		SuiteTable::Actions => {
			RowTable::<CoachAction>::new(store.clone())
				.remove(key)
				.await?
		}
		SuiteTable::Claims => {
			RowTable::<Claim>::new(store.clone()).remove(key).await?
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
		suite: &LoadedSuite,
		table: SuiteTable,
		key: &str,
	) -> Vec<String> {
		match table {
			SuiteTable::Evals => workspace
				.rubrics()
				.iter()
				.filter(|(_, rubric)| {
					rubric
						.citations()
						.any(|citation| citation.eval.as_str() == key)
				})
				.map(|(reference, _)| reference.to_string())
				.collect(),
			SuiteTable::Rubrics => suite
				.render
				.iter()
				.filter(|spec| spec.rubric.as_str() == key)
				.map(|spec| {
					format!("{}/render/{}", suite.manifest.name, spec.rubric)
				})
				.collect(),
			SuiteTable::Render | SuiteTable::Actions | SuiteTable::Claims => {
				Vec::new()
			}
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
				"eval/drop --suite=acme_biz --table=evals --key=product.origin",
			)
			.await
			.xpect_contains(
				"acme_biz/evals/product.origin is still named by acme_biz/owner",
			);
		fixture
			.refused(
				"eval/drop --suite=acme_course --table=rubrics --key=01-business-plan",
			)
			.await
			.xpect_contains(
				"still named by acme_course/render/01-business-plan",
			);
		fixture
			.refused(
				"eval/drop --suite=acme_biz --table=evals --key=product.nothing",
			)
			.await
			.xpect_contains("is no row");
		// a render spec names nothing, so it goes, and then its rubric may
		fixture
			.ok(
				"eval/drop --suite=acme_course --table=render --key=01-business-plan",
			)
			.await
			.xpect_eq("dropped acme_course/render/01-business-plan\n");
		fixture
			.ok(
				"eval/drop --suite=acme_course --table=rubrics --key=01-business-plan",
			)
			.await
			.xpect_eq("dropped acme_course/rubrics/01-business-plan\n");
		fixture.ok("eval/check").await.xpect_contains("0 error(s)");
	}
}
