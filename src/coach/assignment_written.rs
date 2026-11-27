use crate::prelude::*;
use beet::prelude::*;

/// `<HoldClaim/>`: a coach sitting's first step, holding the claim the
/// sitting is about, the `claim` param of the request its window was
/// entered with, ie `--claim=per-weekend`, and reading the `test` it opens
/// with from the register, before the coach's first turn.
///
/// ```bsx
/// <Thread {(Sequence, RunThread, MountThreadStore{path:"coach/{claim}"})}>
///     ..actors..
///     <Fallback>
///         <Sequence>
///             <HoldClaim/>
///             <Repeat>
///                 <Sequence>
///                     <RepeatWhileFunctionCallOutput><CreateActor name="Coach" kind="Agent" ../></RepeatWhileFunctionCallOutput>
///                     <AssignmentWritten/>
///                     <CreateActor name="Owner" kind="User" {UserInput}/>
///                 </Sequence>
///             </Repeat>
///         </Sequence>
///         <EndWith/>
///     </Fallback>
/// </Thread>
/// ```
///
/// The held claim lands on the window as a [`HeldClaim`], which
/// [`AssignmentWritten`] reads.
///
/// ## Errors
///
/// Errors outside an entered thread, for an entry with no `claim` param,
/// and for a claim the register lacks.
#[action]
#[derive(Debug, Default, Clone, Component, Reflect)]
#[reflect(Component, Default)]
pub async fn HoldClaim(cx: ActionContext) -> Result<Outcome> {
	let (window, claim) = cx
		.caller
		.with_state::<AncestorQuery<(Entity, &ThreadEntry)>, _>(
			|entity, entries| -> Result<(Entity, SmolStr)> {
				let (window, entry) = entries.get(entity).map_err(|_| {
					bevyhow!("`<HoldClaim/>` runs in an entered thread")
				})?;
				let claim =
					entry.parts().get_param("claim").ok_or_else(|| {
						bevyhow!(
							"`<HoldClaim/>` reads the `claim` param of the request \
						 its sitting was entered with, ie `--claim=<claim id>`"
						)
					})?;
				(window, SmolStr::from(claim)).xok()
			},
		)
		.await??;
	let opened = HeldClaim::test_of(&cx.caller, &claim).await?;
	cx.caller
		.world()
		.entity(window)
		.insert(HeldClaim { claim, opened })
		.await?;
	Outcome::PASS.xok()
}

/// `<AssignmentWritten/>`: a coach sitting's end condition, passing while
/// the claim [`HoldClaim`] holds keeps the `test` it opened with and failing
/// once the coach has written a new one, the assignment that ends every
/// sitting.
///
/// ## Errors
///
/// Errors before a [`HoldClaim`] has run in its window.
#[action]
#[derive(Debug, Default, Clone, Component, Reflect)]
#[reflect(Component, Default)]
pub async fn AssignmentWritten(cx: ActionContext) -> Result<Outcome> {
	let held = cx
		.caller
		.with_state::<AncestorQuery<&HeldClaim>, _>(|entity, held| {
			held.get(entity).cloned().map_err(|_| {
				bevyhow!("`<AssignmentWritten/>` runs after a `<HoldClaim/>`")
			})
		})
		.await??;
	match HeldClaim::test_of(&cx.caller, &held.claim).await? == held.opened {
		true => Outcome::PASS,
		false => Outcome::FAIL,
	}
	.xok()
}

/// The claim a coach sitting holds, on its window: its id and the `test` it
/// opened with, written by [`HoldClaim`].
#[derive(Debug, Clone, Component)]
pub struct HeldClaim {
	/// The claim's id.
	pub claim: SmolStr,
	/// Its `test` as the sitting opened.
	pub opened: String,
}

impl HeldClaim {
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
		RowTable::<Claim>::new(store.with_subdir(RelPath::new("packages/acme")))
			.get(id)
			.await
			.unwrap()
	}

	/// The sittings stored under the coach's threads about the held claim.
	async fn sittings(store: &BlobStore) -> Vec<Vec<String>> {
		Fixture::sittings(store, "coach/per-weekend").await
	}

	/// `coach.bsx` with the held claim raised, as `next` names its sitting.
	async fn coach() -> FixtureEntry {
		let mut entry = FixtureEntry::of("coach.bsx").await;
		entry
			.request(raise_claim())
			.status()
			.xpect_eq(StatusCode::OK);
		entry
	}

	/// The start request of a sitting on `claim`, `--claim=<claim>`.
	fn sit(claim: &str) -> Request {
		Request::get("/").with_param("claim", claim)
	}

	/// A sitting: it adds a claim, raises the held one's evidence and
	/// writes its assignment, which ends it as a pass, its thread stored
	/// under `coach/per-weekend` and `next` on the claim it added.
	#[beet::test(timeout_ms = 120000)]
	async fn a_sitting_ends_on_its_assignment() {
		let mut entry = coach().await;
		next(&mut entry).xpect_starts_with("## Next: coach per-weekend");
		entry
			.start(sit("per-weekend"))
			.unwrap()
			.xpect_eq(Outcome::PASS);
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
		// the seed: the procedure naming the claim held, the actions, the
		// register and the claim's row
		let posts = &sittings[0];
		posts[0].as_str().xpect_starts_with(
			"You are the coach of Acme's owner. Hold the claim per-weekend to",
		);
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
		posts
			.last()
			.unwrap()
			.xpect_eq("Your assignment: ask three traders on Saturday to book the next weekend at $120.");
		next(&mut entry).xpect_starts_with("## Next: coach traders-return");
	}

	/// A sitting's mount is filled from the claim it is entered with, so an
	/// entry naming none is refused.
	#[beet::test(timeout_ms = 120000)]
	async fn a_sitting_needs_its_claim() {
		coach()
			.await
			.start(Request::get("/"))
			.unwrap_err()
			.to_string()
			.xpect_contains("names the param `claim`");
	}

	/// The coach's store is the workspace package's assets: a write under
	/// `docs/` lands there and never in the documents, and a `..` above the
	/// scope is dropped by `RelPath`, keeping the write inside it.
	#[beet::test(timeout_ms = 120000)]
	async fn the_coach_writes_only_its_notes() {
		let mut entry = coach().await;
		entry.start(sit("per-weekend")).unwrap();
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

	/// The loop stops at the sitting `next` names, a person's to hold,
	/// saying which claim to sit down with, and runs no coach.
	#[beet::test(timeout_ms = 120000)]
	async fn the_loop_stops_for_a_sitting() {
		let mut entry = FixtureEntry::looping(3).await;
		entry
			.request(raise_claim())
			.status()
			.xpect_eq(StatusCode::OK);
		let response = entry.request(Request::get("iterate"));
		response.status().xpect_eq(StatusCode::OK);
		async_ext::block_on(response.text())
			.unwrap()
			.xpect_starts_with("coach per-weekend:");
		sittings(&entry.store).await.len().xpect_eq(0);
		claim(&entry.store, "per-weekend")
			.await
			.status
			.xpect_eq(ClaimStatus::Open);
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
