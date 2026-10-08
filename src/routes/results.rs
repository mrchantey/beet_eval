use super::*;
use crate::prelude::*;
use beet::prelude::*;

/// `eval/results`: runs every check, merges the grades and reads every
/// rubric, writing `results/summary.json`; answers with the run, the
/// [`ResultsReport`] for a markup `Accept` and the stored [`Results`] for a
/// serde one.
#[action(route = "results")]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
pub async fn EvalResults(
	cx: ActionContext<Request>,
) -> Result<DataPage<Results>> {
	let run = Run::of(&cx.caller).await?;
	let summary = RelPath::new(Results::SUMMARY);
	json_ext::write(&run.workspace.results(), &summary, &run.results).await?;
	let written =
		format!("{}/{}", run.workspace.manifest.results, Results::SUMMARY);
	let judged = run.workspace.judged();
	let report = rsx! {
		<ResultsReport
			results=run.results.clone()
			judged=judged
			manifest=run.workspace.manifest.clone()
			written=written
		/>
	};
	DataPage::new(&cx.caller, report, run.results).await
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
			.xpect_starts_with(
				"# Results\n\n13 of 14 checks pass; no grades; 2 rubrics read; results/summary.json written.\n",
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
			.ok("eval/results")
			.await
			.xpect_contains(
				"| `structure.price-list-block` | pass | 2 rows at docs/product.md:19 |",
			)
			.xpect_contains(
				"| `acme_course/01-business-plan` | 2 | 0 | 0 | 2 |",
			)
			.xpect_contains("- `structure.no-open-asks` needs 2, check fails");
		// a tool reads the stored form
		MediaType::Json
			.deserialize::<Results>(
				fixture
					.ok("eval/results --accept=application/json")
					.await
					.as_bytes(),
			)
			.unwrap()
			.rubrics
			.len()
			.xpect_eq(2);
	}
}
