use super::*;
use crate::prelude::*;
use beet::prelude::*;

/// `eval/put`: writes one row of a package's table, refused unless it parses
/// as the table's type and keeps the law `eval/check` holds every row to: an
/// eval's prose, levels, anchors and check params, an id no other package
/// defines, a rubric's citations of known evals at levels they reach, a coach
/// action's served sections and a claim's slug and block. How a person or an
/// agent adds or changes an eval, a rubric, a render spec, a coach action or
/// a claim. The row is given inline with `--row`, its JSON text, which an
/// agent's `<RouteTool>` call sends from the object it passes, or drafted into
/// the workspace store and named with `--from`.
#[action(route = "put")]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
pub async fn EvalPut(
	/// The package whose table takes the row.
	#[param]
	package: SmolStr,
	/// The table the row is written to.
	#[param]
	table: PackageTable,
	/// The row in its stored form, replacing any of the same key.
	#[param]
	row: Option<Value>,
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
	let store = &loaded.store;
	let row = match (row, from) {
		// a command line's text that is no JSON object, refused as the JSON
		// it fails to be
		(Some(Value::Str(text)), None) => {
			MediaType::Json.deserialize::<Value>(text.as_bytes())?
		}
		(Some(row), None) => row,
		(None, Some(from)) => MediaType::Json
			.deserialize::<Value>(&workspace.store.get(&from).await?)?,
		_ => bevybail!("give the row with exactly one of `--row` and `--from`"),
	};
	let outline = workspace.outline().ok();
	let problems = match table {
		PackageTable::Evals => {
			let eval = row.into_serde::<Eval>()?;
			let mut problems = Laws::eval_row(&cx.caller, &eval, outline).await;
			if let Some(other) = workspace
				.eval(&eval.id)
				.filter(|existing| existing.package != loaded.manifest.name)
			{
				problems.push(format!(
					"{} is already defined in {}",
					eval.id, other.package
				));
			}
			Put::finish(
				problems,
				RowTable::<Eval>::new(store.clone()).push(eval),
			)
			.await?
		}
		PackageTable::Rubrics => {
			let rubric = row.into_serde::<Rubric>()?;
			let problems = Laws::rubric(&workspace, &rubric);
			Put::finish(
				problems,
				RowTable::<Rubric>::new(store.clone()).push(rubric),
			)
			.await?
		}
		PackageTable::Render => {
			let spec = row.into_serde::<RenderSpec>()?;
			let problems = Laws::prose(&spec);
			Put::finish(
				problems,
				RowTable::<RenderSpec>::new(store.clone()).push(spec),
			)
			.await?
		}
		PackageTable::Actions => {
			let action = row.into_serde::<CoachAction>()?;
			let problems = Laws::action(&action, outline);
			Put::finish(
				problems,
				RowTable::<CoachAction>::new(store.clone()).push(action),
			)
			.await?
		}
		PackageTable::Claims => {
			let claim = row.into_serde::<Claim>()?;
			let problems = Laws::claim(&claim, outline);
			Put::finish(
				problems,
				RowTable::<Claim>::new(store.clone()).push(claim),
			)
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
					&format!(
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

	/// A claim put as an agent's tool call puts it, its row JSON text, is
	/// held to the claim law: a slug id, one sentence, and a block the outline
	/// declares; a claim the law refuses is never written, and `eval/drop`
	/// retires one.
	#[beet::test]
	async fn puts_claims_that_keep_the_law() {
		let mut fixture = Fixture::new().await;
		let put = |id: &str, block: &str, evidence: u8| {
			Fixture::request("eval/put --package=acme --table=claims")
				.with_param(
					"row",
					&format!(
						r#"{{"id":"{id}","claim":"Traders pay per weekend.",
						"block":"{block}","evidence":{evidence},"stakes":"Fatal",
						"status":"Open","test":"","sources":[]}}"#
					),
				)
		};
		fixture
			.answer(put("per-weekend", "product#pricing", 1))
			.await
			.xpect_eq((true, "put acme/claims\n".to_string()));
		let (ok, text) = fixture
			.answer(put("Per Weekend", "product#nowhere", 1))
			.await;
		ok.xpect_false();
		text.xpect_contains("must be a slug").xpect_contains(
			"block product#nowhere: names a section the outline does not declare",
		);
		let (ok, text) =
			fixture.answer(put("too-sure", "product#pricing", 4)).await;
		ok.xpect_false();
		text.xpect_contains("evidence 4 is not 0 to 3");
		fixture
			.ok("eval/rows --package=acme --table=claims")
			.await
			.xpect_contains("1 row")
			.xpect_contains("### per-weekend");
		fixture
			.ok("eval/drop --package=acme --table=claims --key=per-weekend")
			.await
			.xpect_eq("dropped acme/claims/per-weekend\n");
	}
}
