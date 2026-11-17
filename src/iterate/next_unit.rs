use crate::prelude::*;
use beet::prelude::*;

/// One turn of the loop: computes the [`NextStep`] of the workspace above it
/// and requests the unit of work it names of the url space it is a route of,
/// `<verb>/<first target>`, ie `grade/product` or
/// `build/acme_course/01-business-plan`, the role's route building a thread
/// for that unit. Fails on `done`, the loop's break.
///
/// ```bsx
/// <Route path="iterate" {ExchangeSequence}>
///     <RepeatTimes total_times=3 {OutcomeOverload}><NextUnit/></RepeatTimes>
/// </Route>
/// <Route path="grade/:document" {ThreadRoute{template:"GraderThread"}}/>
/// ```
///
/// A verb with no route errors with the router's own not-found, so a loop
/// missing a role stops rather than skipping it, and so does a role whose
/// thread fails.
#[action]
#[derive(Debug, Default, Clone, Component, Reflect)]
#[reflect(Component, Default)]
pub async fn NextUnit(cx: ActionContext) -> Result<Outcome> {
	let step = NextStep::of(&cx.caller).await?;
	NextUnit::take(&cx.caller, &step).await
}

impl NextUnit {
	/// The request running `step`, `<verb>/<first target>`, none when done.
	pub fn path(step: &NextStep) -> Option<String> {
		match (&step.verb, step.targets.first()) {
			(NextVerb::Done, _) => None,
			(verb, Some(target)) => Some(format!("{}/{target}", verb.word())),
			(verb, None) => Some(verb.word().to_string()),
		}
	}

	/// Take `step` from `caller`: request its unit of the url space `caller`
	/// is a route of, passing once it answers, failing on `done`.
	async fn take(caller: &AsyncEntity, step: &NextStep) -> Result<Outcome> {
		let Some(path) = Self::path(step) else {
			info!("next: done");
			return Outcome::FAIL.xok();
		};
		info!("next: {path}");
		let response = RouteCall::new(&path).exchange(caller).await?;
		let status = response.status();
		let text = response.text().await?;
		if !status.is_ok() {
			bevybail!("the unit `{path}` answered {status}: {text}");
		}
		debug!("{path}: {text}");
		Outcome::PASS.xok()
	}
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	/// A step naming `verb` on `targets`.
	fn step(verb: NextVerb, targets: &[&str]) -> NextStep {
		NextStep {
			verb,
			targets: targets
				.iter()
				.map(|target| SmolStr::new(*target))
				.collect(),
			why: default(),
			documents: default(),
			rubrics: default(),
		}
	}

	/// A `Repeat` over a turn taking `next` in a url space with no routes,
	/// answering the loop's outcome.
	async fn repeat(next: NextStep) -> Result<Outcome> {
		let mut world = (AsyncPlugin, RouterPlugin).into_world();
		let router = world.spawn(Router).id();
		let repeat = world.spawn((ChildOf(router), Repeat::new())).id();
		world.spawn((
			ChildOf(repeat),
			Action::<(), Outcome>::new_async(
				async move |cx: ActionContext| -> Result<Outcome> {
					NextUnit::take(&cx.caller, &next).await
				},
			),
		));
		world.entity_mut(repeat).call::<(), Outcome>(()).await
	}

	#[beet::test]
	fn a_unit_is_its_verb_and_first_target() {
		NextUnit::path(&step(NextVerb::Grade, &["product", "market"]))
			.xpect_eq(Some("grade/product".to_string()));
		NextUnit::path(&step(NextVerb::Build, &[
			"acme_course/01-business-plan",
		]))
		.xpect_eq(Some("build/acme_course/01-business-plan".to_string()));
		NextUnit::path(&step(NextVerb::Done, &[])).xpect_none();
	}

	/// `done` breaks the loop, requesting nothing: a `Repeat` never passes,
	/// so its only way out is a turn that fails.
	#[beet::test]
	async fn done_breaks_the_loop() {
		repeat(step(NextVerb::Done, &[]))
			.await
			.unwrap()
			.xpect_eq(Outcome::FAIL);
	}

	/// A verb with no route stops the loop with the router's not-found,
	/// rather than skipping the unit.
	#[beet::test]
	async fn a_verb_with_no_route_fails_loudly() {
		repeat(step(NextVerb::Scaffold, &["docs/"]))
			.await
			.unwrap_err()
			.to_string()
			.xpect_contains("the unit `scaffold/docs/` answered 404 Not Found");
	}
}
