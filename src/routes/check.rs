use super::*;
use crate::prelude::*;
use beet::prelude::*;

/// `eval/check`: every table of every package against the law: rows that
/// parse, ids unique across the workspace, the prose rules, each check's
/// params against its route's, citations of known evals at levels they can
/// reach, and every anchor, block home and agreement against the outline.
/// Answers with each problem and a count line, exiting 1 on any.
#[action(route = "check")]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
pub async fn EvalCheck(
	/// Also list the evals no rubric cites.
	#[param]
	unused: bool,
	cx: ActionContext<Request>,
) -> Result<DataPage<CheckSummary>> {
	let workspace = LoadedWorkspace::of(&cx.caller).await?;
	let mut errors = workspace.problems.clone();
	if let Err(err) = workspace.document_package() {
		errors.push(err.to_string());
	}
	let outline = workspace.outline().ok();
	for packaged in &workspace.evals {
		let at = format!("{}/evals/{}", packaged.package, packaged.eval.id);
		for problem in Laws::eval_row(&cx.caller, &packaged.eval, outline).await
		{
			errors.push(format!("{at}: {problem}"));
		}
	}
	if let Some(outline) = outline {
		for block in &outline.blocks {
			if let Some(problem) = Laws::address(outline, &block.lives_in) {
				errors
					.push(format!("outline: block {}: {problem}", block.name));
			}
		}
	}
	let mut cited = HashMap::<EvalId, usize>::default();
	for (reference, rubric) in workspace.rubrics() {
		for citation in rubric.citations() {
			*cited.entry(citation.eval.clone()).or_default() += 1;
		}
		for problem in Laws::rubric(&workspace, rubric) {
			errors.push(format!("{reference}: {problem}"));
		}
	}
	for package in &workspace.packages {
		for spec in &package.render {
			for problem in Laws::prose(spec) {
				errors.push(format!(
					"{}/render/{}: {problem}",
					package.manifest.name, spec.rubric
				));
			}
		}
		for action in &package.actions {
			for problem in Laws::prose(action) {
				errors.push(format!(
					"{}/actions/{}: {problem}",
					package.manifest.name, action.id
				));
			}
		}
	}
	let checked = workspace
		.evals
		.iter()
		.filter(|packaged| packaged.eval.is_checked())
		.count();
	let summary = CheckSummary {
		evals: workspace.evals.len(),
		judged: workspace.evals.len() - checked,
		checked,
		packages: workspace.packages.len(),
		citations: cited.values().sum::<usize>(),
		uncited: workspace
			.evals
			.iter()
			.filter(|packaged| !cited.contains_key(&packaged.eval.id))
			.map(|packaged| packaged.eval.id.clone())
			.collect(),
		errors,
	};
	let status = match summary.errors.is_empty() {
		true => StatusCode::OK,
		false => StatusCode::UNPROCESSABLE_CONTENT,
	};
	let report = rsx! { <CheckReport summary=summary.clone() unused=unused/> };
	DataPage::new(&cx.caller, report, summary)
		.await?
		.with_status(status)
		.xok()
}

/// What `eval/check` found: every problem, and what the workspace holds.
#[derive(Debug, Default, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct CheckSummary {
	/// Every law a row breaks, one line each.
	pub errors: Vec<String>,
	/// Every eval of every package.
	pub evals: usize,
	/// The judged ones.
	pub judged: usize,
	/// The checked ones.
	pub checked: usize,
	/// The packages.
	pub packages: usize,
	/// Every citation of every rubric.
	pub citations: usize,
	/// The evals no rubric cites.
	pub uncited: Vec<EvalId>,
}

impl CheckSummary {
	/// The count line, ie `19 evals (5 judged, 14 checked) in 3 packages`.
	pub fn line(&self) -> String {
		format!(
			"{} evals ({} judged, {} checked) in {} packages, {} citations, {} uncited, {} error(s)",
			self.evals,
			self.judged,
			self.checked,
			self.packages,
			self.citations,
			self.uncited.len(),
			self.errors.len()
		)
	}
}

/// The check as a person reads it: every problem, the count line, and with
/// `unused` the evals no rubric cites.
#[template]
pub fn CheckReport(summary: CheckSummary, unused: bool) -> impl Bundle {
	let errors = (!summary.errors.is_empty()).then(|| {
		let items = summary
			.errors
			.iter()
			.map(|error| rsx! { <li>{error.clone()}</li> })
			.collect::<Vec<_>>();
		rsx! { <ul>{items}</ul> }
	});
	let uncited = (unused && !summary.uncited.is_empty()).then(|| {
		let items = summary
			.uncited
			.iter()
			.map(|id| rsx! { <li>{format!("uncited: {id}")}</li> })
			.collect::<Vec<_>>();
		rsx! { <ul>{items}</ul> }
	});
	rsx! {
		{errors}
		<p>{summary.line()}</p>
		{uncited}
	}
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	#[beet::test]
	async fn checks_the_fixture_clean() {
		Fixture::new()
			.await
			.ok("eval/check --unused")
			.await
			.xpect_eq(
				"19 evals (5 judged, 14 checked) in 3 packages, 21 citations, 0 uncited, 0 error(s)\n",
			);
	}

	/// The fixture document package's evals.
	fn evals(fixture: &Fixture) -> beet::net::prelude::Table<Eval> {
		TableStore::new(
			fixture.store.with_subdir(RelPath::new("packages/acme_biz")),
		)
		.table::<Eval>()
	}

	/// Every law a row breaks is named, and the verb exits 1.
	#[beet::test]
	async fn refuses_broken_rows() {
		let mut fixture = Fixture::new().await;
		let evals = evals(&fixture);
		let mut eval = evals.get("product.origin").await.unwrap();
		eval.statement = "Two sentences. Not one.".into();
		eval.anchor = Some(Address::parse("product#nowhere").unwrap());
		evals.push(eval).await.unwrap();
		fixture
			.refused("eval/check")
			.await
			.xpect_contains(
				"acme_biz/evals/product.origin: the statement must be exactly one sentence",
			)
			.xpect_contains(
				"anchor product#nowhere: names a section the outline does not declare",
			)
			.xpect_contains("2 error(s)");
	}

	/// A check's params are refused as its route would refuse them as a
	/// complete request, before any run: a key it does not take, a value its
	/// params type cannot read, a required param left out.
	#[beet::test]
	async fn refuses_params_the_check_route_cannot_take() {
		for (params, refusal) in [
			(
				r#"{"document": "index", "part": "Title", "section": "brand#name-and-tagline", "surprise": "yes"}"#,
				"structure.index-name-agreed: check/agrees: unknown param `--surprise`",
			),
			(
				r#"{"document": "index", "part": "Subtitle", "section": "brand#name-and-tagline"}"#,
				"structure.index-name-agreed: check/agrees: invalid param `--part`",
			),
			(
				r#"{"document": "index", "part": "Title"}"#,
				"structure.index-name-agreed: check/agrees: missing required param `--section`",
			),
		] {
			let mut fixture = Fixture::new().await;
			let evals = evals(&fixture);
			let mut agreed =
				evals.get("structure.index-name-agreed").await.unwrap();
			if let Levels::Binary { check: Some(check) } = &mut agreed.levels {
				check.params = MediaBytes::new_json(params)
					.deserialize::<Value>()
					.unwrap();
			}
			evals.push(agreed).await.unwrap();
			fixture.refused("eval/check").await.xpect_contains(refusal);
		}
	}
}
