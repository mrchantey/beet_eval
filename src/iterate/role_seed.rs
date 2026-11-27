use crate::prelude::*;
use beet::prelude::*;

/// `<RoleSeed role="clerk"/>`: the seed post holding a role's procedure, the
/// file `seeds/<role>.md` of the workspace's document package, read by the
/// entry creating the thread, so the procedure has one source, every package
/// ships its own, and an edited procedure reaches a new thread only.
///
/// Each `{param}` of the procedure is filled from the request entering, ie
/// the coach's `{claim}`, and `{{` and `}}` spell a literal brace
/// ([`ThreadEntry::fill`]).
///
/// The package is the one the workspace manifest names, wherever it lives,
/// beside the workspace or in a bucket, so an entry names a role and never a
/// path.
///
/// ## Errors
///
/// Errors when the package has no seed for the role, and on a param the
/// request lacks.
#[template]
pub fn RoleSeed(
	/// The role, ie `clerk`, `grader`, `builder` or `coach`.
	#[prop(required)]
	role: SmolStr,
) -> impl Bundle {
	SeedSource::new(
		format!("<RoleSeed role=\"{role}\">"),
		Action::new_async(
			async move |cx: ActionContext<ThreadEntry>| -> Result<String> {
				let workspace = LoadedWorkspace::of(&cx.caller).await?;
				let package = workspace.document_package()?;
				let path = RelPath::new(format!("seeds/{role}.md"));
				let bytes = package.store.get(&path).await.map_err(|err| {
					bevyhow!(
						"the document package {} has no seed for the {role}: {err}",
						package.manifest.name
					)
				})?;
				cx.input.fill(&String::from_utf8(bytes.to_vec())?)
			},
		),
	)
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	/// The clerk's threads about the product.
	const CLERK: &str = "clerk/product";
	/// The builder's threads about the course's business plan.
	const BUILDER: &str = "builder/acme_course/01-business-plan";

	/// The texts of every sitting persisted under `dir` in `store`.
	async fn sittings(store: &BlobStore, dir: &str) -> Vec<Vec<String>> {
		Fixture::sittings(store, dir).await
	}

	/// The clerk's turn: its seed is its procedure from the document package
	/// and the unit `next` requested of its route, it reads the results, edits the document,
	/// hears the owner, and leaves `next` on the next unit. One unit a turn:
	/// no other role ran.
	#[beet::test(timeout_ms = 120000)]
	async fn the_clerk_writes_one_unit() {
		let (store, response) = Fixture::iterate(1).await;
		response.status().xpect_eq(StatusCode::OK);
		let clerk = sittings(&store, CLERK).await;
		clerk.len().xpect_eq(1);
		let sitting = &clerk[0];
		sitting[0]
			.as_str()
			.xpect_starts_with("You are the clerk of Acme's");
		sitting[1].xpect_eq("write product");
		sitting[2].xpect_eq("call eval/results");
		sitting
			.iter()
			.any(|text| text == "call edit-text")
			.xpect_true();
		sitting.last().unwrap().xpect_eq("Per weekend, as written.");
		Fixture::over(store.clone())
			.read("docs/product.md")
			.await
			.xpect_contains("We charge per weekend")
			.xnot()
			.xpect_contains("TODO(ask");
		Fixture::over(store.clone())
			.ok("eval/next")
			.await
			.xpect_starts_with("## Next: grade product");
		sittings(&store, "grader").await.len().xpect_eq(0);
		sittings(&store, BUILDER).await.len().xpect_eq(0);
	}

	/// The grader's turn: it grades through `grade` alone, is refused a quote
	/// the page does not hold and a write its router has no route for, and
	/// leaves the document as it found it with every judged eval graded.
	#[beet::test(timeout_ms = 120000)]
	async fn the_grader_grades_one_unit() {
		let (store, response) = Fixture::iterate(2).await;
		response.status().xpect_eq(StatusCode::OK);
		let grader = sittings(&store, "grader").await;
		grader.len().xpect_eq(1);
		let outputs = grader[0]
			.iter()
			.filter(|text| text.starts_with("output "))
			.cloned()
			.collect::<Vec<_>>();
		// its own router answers the write it has no route for
		outputs[1]
			.as_str()
			.xpect_contains("404 Not Found")
			.xpect_contains("/write-text");
		outputs[2]
			.as_str()
			.xpect_contains("not verbatim from product#idea");
		let mut fixture = Fixture::over(store.clone());
		fixture
			.read("docs/product.md")
			.await
			.xpect_contains("We charge per weekend");
		let grades =
			RowTable::<Grade>::new(store.with_subdir(RelPath::new("results")));
		grades
			.get("product.origin")
			.await
			.unwrap()
			.level
			.xpect_eq(EvalLevel::SOUND);
		grades
			.get("product.quote-held")
			.await
			.unwrap()
			.level
			.xpect_eq(EvalLevel::MISSING);
		fixture
			.ok("eval/next")
			.await
			.xpect_starts_with("## Next: build acme_course/01-business-plan");
	}

	/// The builder's sittings: the first writes the form's answers in the
	/// build directory, the only store it can write, and its build is refused
	/// while they await review; once a person approves them, the next lands
	/// its edit in the form's parts and the build lists the built form's
	/// cells.
	#[beet::test(timeout_ms = 120000)]
	async fn the_builder_builds_one_unit() {
		let (store, response) = Fixture::iterate(3).await;
		response.status().xpect_eq(StatusCode::OK);
		sittings(&store, BUILDER).await.len().xpect_eq(1);
		let mut fixture = Fixture::over(store.clone());
		fixture
			.read(Fixture::ANSWERS)
			.await
			.xpect_contains("Acme Stalls");
		store
			.exists(&RelPath::new("dist/acme_course/01-business-plan.cells.md"))
			.await
			.unwrap()
			.xpect_false();
		fixture.ok("eval/next").await.xpect_contains(
			"`eval/answers acme_course/01-business-plan` await a person's review",
		);
		fixture
			.ok("eval/approve acme_course/01-business-plan --by=ada")
			.await;

		FixtureEntry::over(store.clone())
			.await
			.request(Request::get("build/acme_course/01-business-plan"))
			.status()
			.xpect_eq(StatusCode::OK);
		// the builder continues its one thread about the rubric
		let sittings = sittings(&store, BUILDER).await;
		sittings.len().xpect_eq(1);
		sittings[0]
			.iter()
			.filter(|post| *post == "build acme_course/01-business-plan")
			.count()
			.xpect_eq(2);
		fixture
			.read("dist/acme_course/01-business-plan/word/document.xml")
			.await
			.xpect_contains("Acme Stalls");
		store
			.exists(&RelPath::new(
				"acme_course/01-business-plan/word/document.xml",
			))
			.await
			.unwrap()
			.xpect_false();
		fixture
			.read("dist/acme_course/01-business-plan.cells.md")
			.await
			.xpect_contains("| t0/1,0 | Acme Stalls |");
	}

	/// A role runs by hand through its route: the clerk continues its thread
	/// about one document, while the grader judges fresh every time,
	/// unanchored by its earlier grades.
	#[beet::test(timeout_ms = 120000)]
	async fn a_role_runs_by_hand() {
		let mut entry = FixtureEntry::new().await;
		for path in [
			"grade/product",
			"grade/product",
			"write/product",
			"write/product",
		] {
			entry
				.request(Request::get(path))
				.status()
				.xpect_eq(StatusCode::OK);
		}
		let grader = sittings(&entry.store, "grader").await;
		grader.len().xpect_eq(2);
		for sitting in &grader {
			sitting[1].xpect_eq("grade product");
		}
		let clerk = sittings(&entry.store, CLERK).await;
		clerk.len().xpect_eq(1);
		clerk[0]
			.iter()
			.filter(|text| *text == "Per weekend, as written.")
			.count()
			.xpect_eq(2);
		// a run by hand names no decision
		Fixture::threads(&entry.store, CLERK).await[0]
			.0
			.spawned_by()
			.xpect_none();
	}

	/// Every run of the loop continues one thread, the workspace's journal:
	/// each turn's decision a post by its Loop actor, each unit thread
	/// spawned by its decision, and a second run the Loop actor's second
	/// session of it.
	#[beet::test(timeout_ms = 120000)]
	async fn the_loop_journals_each_decision() {
		let mut entry = FixtureEntry::looping(3).await;
		entry
			.request(Request::get("iterate"))
			.status()
			.xpect_eq(StatusCode::OK);
		let journal = Fixture::threads(&entry.store, "loop").await;
		journal.len().xpect_eq(1);
		let (loop_thread, posts) = &journal[0];
		let decisions = posts
			.iter()
			.filter(|post| {
				matches!(post.as_agent_post(), Some(AgentPost::Text(_)))
			})
			.collect::<Vec<_>>();
		decisions
			.iter()
			.map(|post| post.to_string())
			.collect::<Vec<_>>()
			.iter()
			.zip([
				"write product",
				"grade product",
				"build acme_course/01-business-plan",
			])
			.all(|(decision, unit)| decision.starts_with(unit))
			.xpect_true();
		// each decision spawned the one unit thread it asked for
		for (dir, decision) in [CLERK, "grader", BUILDER].iter().zip(&decisions)
		{
			let units = Fixture::threads(&entry.store, dir).await;
			units.len().xpect_eq(1);
			units[0]
				.0
				.spawned_by()
				.xpect_eq(Some(&PostRef::new(loop_thread.id(), decision.id())));
		}
		// a second run is the Loop actor's second join of the same journal
		let looper = decisions[0].author().clone();
		entry.request(Request::get("iterate"));
		let journal = Fixture::threads(&entry.store, "loop").await;
		journal.len().xpect_eq(1);
		journal[0]
			.1
			.iter()
			.filter(|post| {
				post.author() == &looper
					&& post.log::<SessionOpened>().is_some()
			})
			.count()
			.xpect_eq(2);
	}
}
