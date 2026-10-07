use super::*;
use crate::prelude::*;
use beet::prelude::*;

/// Request params for [`EvalGrade`], surfaced in `--help`.
#[derive(Reflect)]
struct GradeParams {
	/// The judged eval graded, ie `market.competitors-named`.
	eval: EvalId,
	/// The level by the eval's own lines, 0 to 3, the lower of two when
	/// between them.
	level: u8,
	/// Where the evidence sits; defaults to the eval's anchor.
	anchor: Option<Address>,
	/// A verbatim quote of at most twenty-five words from the anchor, absent
	/// only at level 0 when nothing is written there.
	evidence: Option<SmolStr>,
	/// The grader of record, a model or a person.
	by: SmolStr,
}

/// `eval/grade`: writes one [`Grade`] to the results' `grades` table, dated
/// today, replacing any earlier grade of the eval. Refused unless the eval
/// is known and judged, the level is one it admits and at most 1 where an
/// ask is open, and the evidence, required above 0, is at most twenty-five
/// words and verbatim from the anchored text, whitespace aside.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
#[require(
	PathPartial = PathPartial::new("grade"),
	ParamsPartial = ParamsPartial::new::<GradeParams>()
)]
pub async fn EvalGrade(cx: ActionContext<Request>) -> Result<Response> {
	let params = cx.input.parse_params::<GradeParams>()?;
	let workspace = LoadedWorkspace::of(&cx.caller).await?;
	workspace.require_clean()?;
	let documents =
		DocumentSet::load(&workspace.docs(), workspace.manifest.docs.clone())
			.await?;
	let grade = match Grading::check(&workspace, &documents, params) {
		Ok(grade) => grade,
		Err(err) => return refusal(format!("{err}\n")).xok(),
	};
	TableStore::new(workspace.results())
		.table::<Grade>()
		.push(grade.clone())
		.await?;
	Response::ok_text(format!(
		"graded {} at {} by {}: {}\n",
		grade.eval, grade.anchor, grade.by, grade.level
	))
	.xok()
}

/// The rules a grade is written under.
struct Grading;

impl Grading {
	fn check(
		workspace: &LoadedWorkspace,
		documents: &DocumentSet,
		params: GradeParams,
	) -> Result<Grade> {
		let id = params.eval;
		let eval = &workspace
			.eval(&id)
			.ok_or_else(|| bevyhow!("unknown eval {id}"))?
			.eval;
		if eval.is_checked() {
			bevybail!(
				"{id} is a checked eval; its result comes from the check"
			);
		}
		let level = EvalLevel::new(params.level)?;
		if !eval.levels.admits(level) {
			bevybail!("{id} is binary and takes 0 or 2, not {level}");
		}
		let anchor =
			params.anchor.unwrap_or_else(|| eval.anchor_or_namespace());
		let document = documents.get(anchor.document_name());
		let text = match (document, anchor.section()) {
			(None, _) => None,
			(Some(document), Some(slug)) => {
				document.section(slug).map(|section| section.body.clone())
			}
			(Some(document), None) => Some(document.text()),
		};
		let open_ask = match (document, anchor.section()) {
			(Some(document), Some(slug)) => document
				.asks
				.iter()
				.any(|ask| ask.section.as_deref() == Some(slug)),
			(Some(document), None) => !document.asks.is_empty(),
			(None, _) => false,
		};
		if open_ask && level > EvalLevel::STATED {
			bevybail!("{anchor} has an ask open, so {id} is graded at most 1");
		}
		let evidence = params
			.evidence
			.map(|evidence| SmolStr::from(evidence.trim()))
			.filter(|evidence| !evidence.is_empty());
		match (&evidence, level) {
			(None, level) if level > EvalLevel::MISSING => {
				bevybail!("a grade above 0 quotes its evidence")
			}
			(Some(quote), _) => {
				let words = quote.split_whitespace().count();
				if words > Grade::MAX_EVIDENCE_WORDS {
					bevybail!(
						"the evidence is {words} words; quote at most {}",
						Grade::MAX_EVIDENCE_WORDS
					);
				}
				let collapse = |text: &str| {
					text.split_whitespace().collect::<Vec<_>>().join(" ")
				};
				let found = text.as_deref().is_some_and(|text| {
					collapse(text).contains(&collapse(quote))
				});
				if !found {
					bevybail!(
						"the evidence is not verbatim from {anchor}: \"{quote}\""
					);
				}
			}
			_ => {}
		}
		Grade {
			eval: id,
			level,
			anchor,
			evidence,
			date: Date::today(),
			by: params.by,
		}
		.xok()
	}
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	#[beet::test]
	async fn writes_a_grade_with_verbatim_evidence() {
		let mut fixture = Fixture::new().await;
		fixture
			.ok("eval/grade --eval=product.alternatives-considered --level=2 --by=a\\ grader --evidence=We\\ weighed\\ a\\ mobile\\ service\\ against\\ the\\ stall")
			.await
			.xpect_eq("graded product.alternatives-considered at product#idea by a grader: 2\n");
		let grade =
			TableStore::new(fixture.store.with_subdir(RelPath::new("results")))
				.table::<Grade>()
				.get("product.alternatives-considered")
				.await
				.unwrap();
		grade.date.xpect_eq(Date::today());
		fixture
			.ok("eval/results")
			.await
			.xpect_contains("1 of 5 judged evals graded");
	}

	#[beet::test]
	async fn refuses_what_the_law_forbids() {
		let mut fixture = Fixture::new().await;
		for (command, refusal) in [
			(
				"--eval=product.origin --level=2 --by=x --evidence=we\\ built\\ a\\ rocket",
				"not verbatim",
			),
			(
				"--eval=product.pricing-justified --level=2 --by=x --evidence=Small\\ stall",
				"graded at most 1",
			),
			(
				"--eval=structure.index-titled --level=2 --by=x",
				"checked eval",
			),
			("--eval=product.quote-held --level=1 --by=x", "binary"),
			(
				"--eval=product.origin --level=2 --by=x",
				"quotes its evidence",
			),
			("--eval=market.nothing --level=0 --by=x", "unknown eval"),
		] {
			fixture
				.refused(&format!("eval/grade {command}"))
				.await
				.xpect_contains(refusal);
		}
	}
}
