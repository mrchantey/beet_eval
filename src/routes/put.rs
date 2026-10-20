use super::*;
use crate::prelude::*;
use beet::prelude::*;

/// `eval/put`: writes one row of a package's table, refused unless it parses
/// as the table's type and keeps the law `eval/check` holds every row to: an
/// eval's prose, levels, anchors and check params, an id no other package
/// defines, a rubric's citations of known evals at levels they reach. How a
/// person or an agent adds or changes an eval, a rubric, a render spec or a
/// coach action. The row is given inline with `--row`, or drafted into the
/// workspace store and named with `--from`.
#[action(route = "put")]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
pub async fn EvalPut(
	/// The package whose table takes the row.
	#[param]
	package: SmolStr,
	/// The table: `evals`, `rubrics`, `render` or `actions`.
	#[param]
	table: PackageTable,
	/// The row in its stored form, JSON, replacing any of the same key.
	#[param]
	row: Option<String>,
	/// The workspace store path of a file holding the row, in place of
	/// `--row`, for a row too long for a command line.
	#[param]
	from: Option<RelPath>,
	cx: ActionContext<Request>,
) -> Result<Response> {
	let workspace = LoadedWorkspace::of(&cx.caller).await?;
	let loaded = workspace.package(&package).ok_or_else(|| {
		bevyhow!("the workspace names no package {}", package)
	})?;
	let tables = TableStore::new(loaded.store.clone());
	let row = match (row, from) {
		(Some(row), None) => row.into_bytes(),
		(None, Some(from)) => workspace.store.get(&from).await?.to_vec(),
		_ => bevybail!("give the row with exactly one of `--row` and `--from`"),
	};
	let row = row.as_slice();
	let problems = match table {
		PackageTable::Evals => {
			let eval = MediaType::Json.deserialize::<Eval>(row)?;
			let mut problems =
				Laws::eval_row(&cx.caller, &eval, workspace.outline().ok())
					.await;
			if let Some(other) = workspace
				.eval(&eval.id)
				.filter(|existing| existing.package != loaded.manifest.name)
			{
				problems.push(format!(
					"{} is already defined in {}",
					eval.id, other.package
				));
			}
			Put::finish(problems, tables.table::<Eval>().push(eval)).await?
		}
		PackageTable::Rubrics => {
			let rubric = MediaType::Json.deserialize::<Rubric>(row)?;
			let problems = Laws::rubric(&workspace, &rubric);
			Put::finish(problems, tables.table::<Rubric>().push(rubric)).await?
		}
		PackageTable::Render => {
			let spec = MediaType::Json.deserialize::<RenderSpec>(row)?;
			let problems = Laws::prose(&spec);
			Put::finish(problems, tables.table::<RenderSpec>().push(spec))
				.await?
		}
		PackageTable::Actions => {
			let action = MediaType::Json.deserialize::<CoachAction>(row)?;
			let problems = Laws::prose(&action);
			Put::finish(problems, tables.table::<CoachAction>().push(action))
				.await?
		}
	};
	match problems.is_empty() {
		true => Response::ok_text(format!("put {}/{}\n", package, table)),
		false => refusal(format!("{}\n", problems.join("\n"))),
	}
	.xok()
}

/// A put written only when its row keeps the law.
struct Put;

impl Put {
	async fn finish(
		problems: Vec<String>,
		write: impl Future<Output = Result>,
	) -> Result<Vec<String>> {
		if problems.is_empty() {
			write.await?;
		}
		problems.xok()
	}
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	#[beet::test]
	async fn puts_rows_that_keep_the_law() {
		let mut fixture = Fixture::new().await;
		let put = |statement: &str| {
			Request::from_cli_str("eval/put")
				.with_param("package", "acme_biz")
				.with_param("table", "evals")
				.with_param(
					"row",
					format!(
						"{{\"id\":\"brand.name-cleared\",\"statement\":\"{statement}\",\
						 \"levels\":\"Generic\",\"sources\":[\"inferred\"],\
						 \"anchor\":\"brand#name-and-tagline\",\"note\":null}}"
					)
					.as_str(),
				)
		};
		let (ok, text) = fixture.answer(put("Cleared. Twice.")).await;
		ok.xpect_false();
		text.xpect_contains("exactly one sentence");
		fixture
			.answer(put("The name is cleared against the register."))
			.await
			.xpect_eq((true, "put acme_biz/evals\n".to_string()));
		fixture
			.ok("eval/check")
			.await
			.xpect_starts_with("20 evals (6 judged, 14 checked)");
	}

	/// A row drafted into the workspace store is put by its path.
	#[beet::test]
	async fn puts_a_row_from_the_store() {
		let mut fixture = Fixture::new().await;
		let mut eval =
			fixture.read("packages/acme_biz/evals/product.origin").await;
		eval = eval.replace("\"Generic\"", "{\"Binary\": {\"check\": null}}");
		fixture
			.store
			.insert(&RelPath::new("drafts/origin.json"), eval)
			.await
			.unwrap();
		fixture
			.ok(
				"eval/put --package=acme_biz --table=evals --from=drafts/origin.json",
			)
			.await
			.xpect_eq("put acme_biz/evals\n");
		fixture
			.read("packages/acme_biz/evals/product.origin")
			.await
			.xpect_contains("\"Binary\"");
		fixture
			.refused("eval/put --package=acme_biz --table=evals")
			.await
			.xpect_contains("exactly one of `--row` and `--from`");
	}
}
