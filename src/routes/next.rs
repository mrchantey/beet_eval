use super::*;
use crate::prelude::*;
use beet::prelude::*;

/// Request params for [`EvalNext`], surfaced in `--help`.
#[derive(Reflect)]
struct NextParams {
	/// `json` for the stored form of the step, rather than its markdown.
	format: Option<ReportFormat>,
}

/// `eval/next`: the next unit of work and why, with the document and rubric
/// tables the choice was made from, computed from the same run as
/// `eval/results`.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
#[require(
	PathPartial = PathPartial::new("next"),
	ParamsPartial = ParamsPartial::new::<NextParams>()
)]
pub async fn EvalNext(cx: ActionContext<Request>) -> Result<Response> {
	let params = cx.input.parse_params::<NextParams>()?;
	let run = Run::of(&cx.caller).await?;
	let dist = run.workspace.dist();
	let mut built = Vec::new();
	for (reference, _) in run.workspace.rubrics() {
		if dist
			.exists(&CellsDump::path(&reference))
			.await
			.unwrap_or(false)
		{
			built.push(reference);
		}
	}
	let step = NextStep::compute(
		&run.workspace,
		&run.documents,
		&run.verdicts,
		&run.results,
		&built,
	)?;
	match params.format.unwrap_or_default() {
		ReportFormat::Md => Response::ok_text(step.to_markdown()),
		ReportFormat::Json => Response::ok_text(json_ext::to_string(&step)?),
	}
	.xok()
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	#[beet::test]
	async fn names_the_next_unit_of_work() {
		let mut fixture = Fixture::new().await;
		fixture
			.ok("eval/next")
			.await
			.xpect_starts_with(
				"## Next: write product\n\n1 ask(s) open and an owner gap of 11, reader gap 4, the most of any document.\n",
			)
			.xpect_contains("| product | yes | 1 | 0 | 11 | 4 | 0/4 | 2026-10-04 |");
		let step = MediaType::Json
			.deserialize::<NextStep>(
				fixture.ok("eval/next --format=json").await.as_bytes(),
			)
			.unwrap();
		step.verb.xpect_eq(NextVerb::Write);
		step.targets.xpect_eq(vec![SmolStr::new("product")]);
	}
}
