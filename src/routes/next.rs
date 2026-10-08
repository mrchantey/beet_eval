use super::*;
use crate::prelude::*;
use beet::prelude::*;

/// `eval/next`: the next unit of work and why, with the document and rubric
/// tables the choice was made from, computed from the same run as
/// `eval/results`: the [`NextReport`] for a markup `Accept` and the stored
/// [`NextStep`] for a serde one.
#[action(route = "next")]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
pub async fn EvalNext(
	cx: ActionContext<Request>,
) -> Result<DataPage<NextStep>> {
	let mut run = Run::of(&cx.caller).await?;
	let dist = run.workspace.dist();
	let mut built = Vec::new();
	for (reference, _) in run.workspace.rubrics() {
		if dist
			.exists(&FillSpec::cells_path(&reference))
			.await
			.unwrap_or(false)
		{
			built.push(reference);
		}
	}
	let step = NextStep::compute(
		&run.workspace,
		&mut run.documents,
		&run.verdicts,
		&run.results,
		&built,
	)?;
	DataPage::new(&cx.caller, rsx! { <NextReport step=step.clone()/> }, step)
		.await
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
				fixture
					.ok("eval/next --accept=application/json")
					.await
					.as_bytes(),
			)
			.unwrap();
		step.verb.xpect_eq(NextVerb::Write);
		step.targets.xpect_eq(vec![SmolStr::new("product")]);
	}
}
