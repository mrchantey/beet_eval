use crate::prelude::*;
use beet::prelude::*;

/// One turn of the loop: computes the [`NextStep`] of the workspace above it,
/// appends the decision to the loop's thread and requests the unit of work it
/// names of the url space it is a route of, `<verb>/<first target>`, ie
/// `grade/product` or `build/acme_course/01-business-plan`, the role's route
/// building a thread for that unit. Fails on `done`, the loop's break.
///
/// ```bsx
/// <Route path="iterate" {ThreadRoute{template:"LoopThread", metadata:{journal:"loop"}}}/>
/// <Route path="grade/:document" {ThreadRoute{template:"GraderThread"}}/>
/// <!-- templates/LoopThread.bsx -->
/// <Thread {(Sequence, MountThreadStore{path:"packages/acme/assets/threads/loop"})}>
///     <CreateActor id=1 name="Loop" kind="Agent" {Fallback}>
///         <RepeatTimes total_times=3><NextUnit/></RepeatTimes>
///         <EndWith/>
///     </CreateActor>
/// </Thread>
/// ```
///
/// Run by an actor of a thread, each decision is a post by that actor,
/// `<verb> <target>: <why>`, and the unit is requested with `spawned_by`
/// naming it, so every unit thread, and every session of one, names the
/// decision that asked for it. Outside a thread it requests the unit alone.
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

	/// The decision `step` is, as the loop's thread records it:
	/// `<verb> <target>: <why>`.
	pub fn decision(step: &NextStep) -> String {
		let unit = std::iter::once(step.verb.word().to_string())
			.chain(step.targets.first().map(ToString::to_string))
			.collect::<Vec<_>>()
			.join(" ");
		match step.why.is_empty() {
			true => unit,
			false => format!("{unit}: {}", step.why),
		}
	}

	/// Take `step` from `caller`: record the decision, then request its unit
	/// of the url space `caller` is a route of, spawned by the decision,
	/// passing once it answers, failing on `done`.
	async fn take(caller: &AsyncEntity, step: &NextStep) -> Result<Outcome> {
		let decision = Self::decide(caller, step).await?;
		let Some(path) = Self::path(step) else {
			info!("next: done");
			return Outcome::FAIL.xok();
		};
		info!("next: {path}");
		let request = match &decision {
			Some(post) => {
				Request::get(&path).with_param("spawned-by", &post.to_string())
			}
			None => Request::get(&path),
		};
		let response = RouteCall::from(request).exchange(caller).await?;
		let status = response.status();
		let text = response.text().await?;
		if !status.is_ok() {
			bevybail!("the unit `{path}` answered {status}: {text}");
		}
		debug!("{path}: {text}");
		Outcome::PASS.xok()
	}

	/// Append `step`'s decision to the thread `caller` runs in, by the actor
	/// running this turn, answering the post: none outside a thread.
	async fn decide(
		caller: &AsyncEntity,
		step: &NextStep,
	) -> Result<Option<PostRef>> {
		let decision = Self::decision(step);
		caller
			.with_state::<(ThreadRecordsQuery, Query<&ChildOf>, Query<&ActorRef>), _>(
				move |entity,
				      (mut thread_records, parents, actors)|
				      -> Result<Option<PostRef>> {
					let Some(actor) = parents
						.iter_ancestors_inclusive(entity)
						.find_map(|ancestor| actors.get(ancestor).ok())
					else {
						return Ok(None);
					};
					let thread = thread_records.thread_id(entity)?;
					let post = AgentPost::new_text(
						**actor,
						thread,
						decision,
						PostStatus::Completed,
					);
					let id = post.id();
					thread_records.push_post(entity, post)?;
					PostRef::new(thread, id).xsome().xok()
				},
			)
			.await?
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
