use super::*;
use crate::prelude::*;
use beet::prelude::*;

/// Request params for [`EvalResults`], surfaced in `--help`.
#[derive(Reflect)]
struct ResultsParams {
	/// Answer with the whole run, `md` for a person or `json` for a tool,
	/// rather than its one line.
	format: Option<ReportFormat>,
}

/// `eval/results`: runs every check, merges the grades and reads every
/// rubric, writing `results/summary.json`; answers with the run's line, or
/// the run itself with `--format`.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
#[require(
	PathPartial = PathPartial::new("results"),
	ParamsPartial = ParamsPartial::new::<ResultsParams>()
)]
pub async fn EvalResults(cx: ActionContext<Request>) -> Result<Response> {
	let params = cx.input.parse_params::<ResultsParams>()?;
	let run = Run::of(&cx.caller).await?;
	let summary = RelPath::new(Results::SUMMARY);
	json_ext::write(&run.workspace.results(), &summary, &run.results).await?;
	let written =
		format!("{}/{}", run.workspace.manifest.results, Results::SUMMARY);
	match params.format {
		None => Response::ok_text(format!(
			"{}; {written} written\n",
			run.results.line(&run.workspace.evals)
		)),
		Some(ReportFormat::Md) => Response::ok_text(
			run.results
				.to_markdown(&run.workspace.evals, &run.workspace.manifest),
		),
		Some(ReportFormat::Json) => {
			Response::ok_text(json_ext::to_string(&run.results)?)
		}
	}
	.xok()
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	#[beet::test]
	async fn runs_and_writes_the_summary() {
		let mut fixture = Fixture::new().await;
		fixture
			.ok("eval/results")
			.await
			.xpect_eq(
				"13 of 14 checks pass; no grades; 2 rubrics read; results/summary.json written\n",
			);
		let summary = json_ext::read::<Results>(
			&fixture.store,
			&RelPath::new("results/summary.json"),
		)
		.await
		.unwrap();
		summary
			.checks
			.iter()
			.find(|check| !check.pass)
			.unwrap()
			.detail
			.clone()
			.xpect_eq("2 open: docs/brand.md:17, docs/product.md:17");
		summary.rubrics[0].id.to_string().xpect_eq("acme_biz/owner");
		fixture
			.ok("eval/results --format=md")
			.await
			.xpect_contains(
				"| `structure.price-list-block` | pass | 2 rows at docs/product.md:19 |",
			)
			.xpect_contains(
				"| `acme_course/01-business-plan` | 2 | 0 | 0 | 2 |",
			)
			.xpect_contains("- `structure.no-open-asks` needs 2, check fails");
	}
}
