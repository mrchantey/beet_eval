use crate::prelude::*;
use beet::prelude::*;

/// `<AssignmentWritten/>`: a coach sitting's end condition, passing while the
/// claim the sitting holds keeps the `test` it opened with and failing once
/// the coach has written a new one, the assignment that ends every sitting.
///
/// ```bsx
/// <Fallback>
///     <Repeat>
///         <Sequence>
///             <RepeatWhileFunctionCallOutput><CreateActor name="Coach" kind="Agent" ../></RepeatWhileFunctionCallOutput>
///             <AssignmentWritten/>
///             <CreateActor name="Owner" kind="User" {UserInput}/>
///         </Sequence>
///     </Repeat>
///     <EndWith/>
/// </Fallback>
/// ```
///
/// The claim is the `claim` param of the request the thread is built for,
/// ie `coach/:claim`, read while the template builds, and its opening `test`
/// is read from the register while the thread builds, before any turn runs.
///
/// ## Errors
///
/// Errors outside a request build, or for a request with no `claim` param.
#[template(system)]
pub fn AssignmentWritten(
	stack: Res<RequestContextStack>,
) -> Result<impl Bundle> {
	let claim = stack
		.try_current()
		.and_then(|cx| cx.parts().get_param("claim").map(SmolStr::from))
		.ok_or_else(|| {
			bevyhow!(
				"`<AssignmentWritten/>` reads the `claim` of the request its \
				 sitting is built for, ie a `coach/:claim` route's"
			)
		})?;
	(
		AssignmentCheck {
			claim: claim.clone(),
			opened: None,
		},
		OpeningTest { claim },
	)
		.xok()
}

/// The action [`AssignmentWritten`] builds: passes while the claim's `test`
/// is `opened`, fails once it differs.
#[action]
#[derive(Debug, Default, Clone, Component, Reflect)]
#[reflect(Component, Default)]
pub async fn AssignmentCheck(
	/// The claim the sitting holds, its id.
	#[field]
	claim: SmolStr,
	/// The claim's `test` as the sitting opened, read while it builds.
	#[field]
	opened: Option<String>,
	cx: ActionContext,
) -> Result<Outcome> {
	let Some(opened) = opened else {
		bevybail!(
			"the sitting on {claim} runs before its opening test was read"
		);
	};
	match AssignmentCheck::test_of(&cx.caller, &claim).await? == opened {
		true => Outcome::PASS,
		false => Outcome::FAIL,
	}
	.xok()
}

impl AssignmentCheck {
	/// The `test` of `claim` in the register of the workspace above
	/// `caller`.
	async fn test_of(caller: &AsyncEntity, claim: &str) -> Result<String> {
		LoadedWorkspace::of(caller)
			.await?
			.claims()
			.into_iter()
			.find(|row| row.id == claim)
			.map(|row| row.test.clone())
			.ok_or_else(|| bevyhow!("the register holds no claim {claim}"))
	}
}

/// Reads the opening `test` of `claim` into the [`AssignmentCheck`] beside
/// it while the thread builds, a pending dependency of the build as a seed
/// is, so it is read before any turn runs, then leaves.
#[derive(Debug, Clone, Component)]
#[component(on_add = OpeningTest::on_add)]
struct OpeningTest {
	claim: SmolStr,
}

impl OpeningTest {
	/// Park the read on the build root and run it as a task holding the
	/// guard, so however it ends the guard resolves.
	fn on_add(mut world: DeferredWorld, cx: HookContext) {
		let entity = cx.entity;
		let root = world.get_resource::<TemplateBuildRoot>().map(|root| **root);
		let Some(claim) = world
			.get::<OpeningTest>(entity)
			.map(|read| read.claim.clone())
		else {
			return;
		};
		world.commands().queue(move |world: &mut World| {
			let guard = TemplatePending::park_on(
				world,
				root.unwrap_or(entity),
				PendingKind::Passive,
				format!("<AssignmentWritten/> reading {claim}"),
			);
			let Ok(mut entity_mut) = world.get_entity_mut(entity) else {
				return;
			};
			entity_mut.run_async_local(async move |entity: AsyncEntity| {
				let root = guard.root();
				let id = entity.id();
				let result = AssignmentCheck::test_of(&entity, &claim).await;
				entity
					.world()
					.with(move |world: &mut World| {
						match result {
							Ok(test) => {
								if let Ok(mut entity) = world.get_entity_mut(id)
								{
									entity.remove::<OpeningTest>();
									if let Some(mut check) =
										entity.get_mut::<AssignmentCheck>()
									{
										check.opened = Some(test);
									}
								}
							}
							Err(err) => {
								if let Ok(mut root) = world.get_entity_mut(root)
								{
									root.insert(TemplateError::new(err));
								}
							}
						}
						guard.resolve(world);
					})
					.await;
			});
		});
	}
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	/// The register's claim the sitting holds: the plan's pricing, fatal and
	/// asserted, resting on the section the fixture's decision ask is in.
	const PER_WEEKEND: &str = r#"{
		"id":"per-weekend","claim":"Traders will pay per weekend rather than by the month.",
		"block":"product#pricing","evidence":0,"stakes":"Fatal","status":"Open",
		"test":"","sources":[]}"#;

	/// `eval/put` of [`PER_WEEKEND`], as the clerk raises a claim.
	fn raise_claim() -> Request {
		Fixture::request("eval/put --package=acme --table=claims")
			.with_param("row", PER_WEEKEND)
	}

	/// What `eval/next` answers of `entry`, as markdown.
	fn next(entry: &mut FixtureEntry) -> String {
		let response = entry.request(
			Request::get("eval/next").with_accept(MediaType::Markdown),
		);
		async_ext::block_on(response.text()).unwrap()
	}

	/// The claim `id` of the register in `store`.
	async fn claim(store: &BlobStore, id: &str) -> Claim {
		TableStore::new(store.with_subdir(RelPath::new("packages/acme")))
			.table::<Claim>()
			.get(id)
			.await
			.unwrap()
	}

	/// The sittings stored under the coach's threads.
	async fn sittings(store: &BlobStore) -> Vec<Vec<String>> {
		Fixture::sittings(store, "coach").await
	}

	/// A sitting run by hand: it adds a claim, raises the held one's evidence
	/// and writes its assignment, which ends it as a pass, its thread stored
	/// under `threads/coach` and `next` on the claim it added.
	#[beet::test(timeout_ms = 120000)]
	async fn a_sitting_runs_by_hand() {
		let mut entry = FixtureEntry::of("coach.bsx").await;
		entry
			.request(raise_claim())
			.status()
			.xpect_eq(StatusCode::OK);
		next(&mut entry).xpect_starts_with("## Next: coach per-weekend");
		let response = entry.request(Request::get("coach/per-weekend"));
		response.status().xpect_eq(StatusCode::OK);
		async_ext::block_on(response.text())
			.unwrap()
			.xpect_eq("Your assignment: ask three traders on Saturday to book the next weekend at $120.");
		let held = claim(&entry.store, "per-weekend").await;
		held.evidence.xpect_eq(Evidence::OBSERVED);
		held.status.xpect_eq(ClaimStatus::Testing);
		held.test.xpect_eq(
			"Ask three traders on Saturday to book the next weekend at $120.",
		);
		claim(&entry.store, "traders-return")
			.await
			.evidence
			.xpect_eq(Evidence::ASSERTED);
		let sittings = sittings(&entry.store).await;
		sittings.len().xpect_eq(1);
		// the seed: the procedure, the actions, the register, the claim held,
		// then the unit
		let posts = &sittings[0];
		posts[0]
			.as_str()
			.xpect_starts_with("You are the coach of Acme's owner.");
		posts[1]
			.as_str()
			.xpect_starts_with("## acme_biz/actions")
			.xpect_contains("### status-quo");
		posts[2]
			.as_str()
			.xpect_starts_with("## acme/claims\n\n1 row");
		posts[3]
			.as_str()
			.xpect_contains("### per-weekend")
			.xpect_contains("- **evidence**: 0");
		posts[4].xpect_eq("coach per-weekend");
		next(&mut entry).xpect_starts_with("## Next: coach traders-return");
	}

	/// The coach's store is the workspace package's assets: a write under
	/// `docs/` lands there and never in the documents, and a `..` above the
	/// scope is dropped by `RelPath`, keeping the write inside it.
	#[beet::test(timeout_ms = 120000)]
	async fn the_coach_writes_only_its_notes() {
		let mut entry = FixtureEntry::of("coach.bsx").await;
		entry.request(raise_claim());
		entry.request(Request::get("coach/per-weekend"));
		let fixture = Fixture::over(entry.store.clone());
		fixture
			.read("packages/acme/assets/docs/product.md")
			.await
			.xpect_eq("Two traders paid for last weekend.");
		fixture
			.read("docs/product.md")
			.await
			.xpect_contains("TODO(ask decision: monthly or per weekend?)");
		fixture
			.read("packages/acme/assets/docs/brand.md")
			.await
			.xpect_eq("escaped");
		fixture
			.read("docs/brand.md")
			.await
			.xnot()
			.xpect_contains("escaped");
	}

	/// The loop requests the sitting `next` names, `coach/<claim id>`.
	#[beet::test(timeout_ms = 120000)]
	async fn the_loop_requests_a_sitting() {
		let mut entry = FixtureEntry::looping(1).await;
		entry
			.request(raise_claim())
			.status()
			.xpect_eq(StatusCode::OK);
		entry
			.request(Request::get("iterate"))
			.status()
			.xpect_eq(StatusCode::OK);
		sittings(&entry.store).await.len().xpect_eq(1);
		claim(&entry.store, "per-weekend")
			.await
			.status
			.xpect_eq(ClaimStatus::Testing);
	}

	/// A decision ask no claim names is the clerk's to raise, so `next` names
	/// a write; once raised, the coach holds the claim.
	#[beet::test]
	async fn a_decision_ask_is_raised_then_coached() {
		let mut fixture = Fixture::new().await;
		fixture
			.ok("eval/next")
			.await
			.xpect_starts_with("## Next: write product")
			.xpect_contains("1 decision ask(s) no claim names");
		fixture.answer(raise_claim()).await.0.xpect_true();
		fixture
			.ok("eval/next")
			.await
			.xpect_starts_with("## Next: coach per-weekend")
			.xpect_contains("per-weekend is fatal at evidence 0 (asserted)");
	}
}
