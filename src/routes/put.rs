use super::*;
use crate::prelude::*;
use beet::prelude::*;

/// Request params for [`EvalPut`], surfaced in `--help`.
#[derive(Reflect)]
struct PutParams {
	/// The package whose table takes the row.
	package: String,
	/// The table: `evals`, `rubrics`, `render` or `actions`.
	table: String,
	/// The row in its stored form, JSON, replacing any of the same key.
	row: Option<String>,
	/// The workspace store path of a file holding the row, in place of
	/// `--row`, for a row too long for a command line.
	from: Option<String>,
}

/// `eval/put`: writes one row of a package's table, refused unless it parses
/// as the table's type and keeps the law `eval/check` holds every row to: an
/// eval's prose, levels, anchors and check params, an id no other package
/// defines, a rubric's citations of known evals at levels they reach. How a
/// person or an agent adds or changes an eval, a rubric, a render spec or a
/// coach action. The row is given inline with `--row`, or drafted into the
/// workspace store and named with `--from`.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
#[require(
	PathPartial = PathPartial::new("put"),
	ParamsPartial = ParamsPartial::new::<PutParams>()
)]
pub async fn EvalPut(cx: ActionContext<Request>) -> Result<Response> {
	let params = cx.input.parse_params::<PutParams>()?;
	let workspace = LoadedWorkspace::of(&cx.caller).await?;
	let package = workspace.package(&params.package).ok_or_else(|| {
		bevyhow!("the workspace names no package {}", params.package)
	})?;
	let tables = TableStore::new(package.store.clone());
	let row = match (params.row, params.from) {
		(Some(row), None) => row.into_bytes(),
		(None, Some(from)) => {
			workspace.store.get(&RelPath::new(&from)).await?.to_vec()
		}
		_ => bevybail!("give the row with exactly one of `--row` and `--from`"),
	};
	let row = row.as_slice();
	let problems = match params.table.as_str() {
		"evals" => {
			let eval = MediaType::Json.deserialize::<Eval>(row)?;
			let mut problems =
				Laws::eval_row(&cx.caller, &eval, workspace.outline().ok())
					.await;
			if let Some(other) = workspace
				.eval(&eval.id)
				.filter(|existing| existing.package != package.manifest.name)
			{
				problems.push(format!(
					"{} is already defined in {}",
					eval.id, other.package
				));
			}
			Put::finish(problems, tables.table::<Eval>().push(eval)).await?
		}
		"rubrics" => {
			let rubric = MediaType::Json.deserialize::<Rubric>(row)?;
			let problems = Laws::rubric(&workspace, &rubric);
			Put::finish(problems, tables.table::<Rubric>().push(rubric)).await?
		}
		"render" => {
			let spec = MediaType::Json.deserialize::<RenderSpec>(row)?;
			let problems = Laws::prose(&spec);
			Put::finish(problems, tables.table::<RenderSpec>().push(spec))
				.await?
		}
		"actions" => {
			let action = MediaType::Json.deserialize::<CoachAction>(row)?;
			let problems = Laws::prose(&action);
			Put::finish(problems, tables.table::<CoachAction>().push(action))
				.await?
		}
		other => bevybail!(
			"`{other}` is no table: expected evals, rubrics, render or actions"
		),
	};
	match problems.is_empty() {
		true => Response::ok_text(format!(
			"put {}/{}\n",
			params.package, params.table
		)),
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
