use crate::prelude::*;
use beet::prelude::*;

/// `<RoleSeed role="clerk"/>`: the seed post holding a role's procedure, the
/// file `seeds/<role>.md` of the workspace's document package, read once
/// while the thread builds, so the procedure has one source, every package
/// ships its own, and an edited procedure is a thread of its own.
///
/// The package is the one the workspace manifest names, wherever it lives,
/// beside the workspace or in a bucket, so an entry names a role and never a
/// path.
#[template]
pub fn RoleSeed(
	/// The role, ie `clerk`, `grader`, `builder` or `coach`.
	#[prop(required)]
	role: SmolStr,
) -> impl Bundle {
	SeedSource::new(
		format!("<RoleSeed role=\"{role}\">"),
		Action::new_async(async move |cx: ActionContext| -> Result<String> {
			let workspace = LoadedWorkspace::of(&cx.caller).await?;
			let package = workspace.document_package()?;
			let path = RelPath::new(format!("seeds/{role}.md"));
			let bytes = package.store.get(&path).await.map_err(|err| {
				bevyhow!(
					"the document package {} has no seed for the {role}: {err}",
					package.manifest.name
				)
			})?;
			String::from_utf8(bytes.to_vec())?.xok()
		}),
	)
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	/// The texts of every sitting the `role` persisted in `store`.
	async fn sittings(store: &BlobStore, role: &str) -> Vec<Vec<String>> {
		Fixture::sittings(store, role).await
	}

	/// The clerk's turn: its seed is its procedure from the document package
	/// and the unit `next` requested of its route, it reads the results, edits the document,
	/// hears the owner, and leaves `next` on the next unit. One unit a turn:
	/// no other role ran.
	#[beet::test(timeout_ms = 120000)]
	async fn the_clerk_writes_one_unit() {
		let (store, response) = Fixture::iterate(1).await;
		response.status().xpect_eq(StatusCode::OK);
		let clerk = sittings(&store, "clerk").await;
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
		sittings(&store, "builder").await.len().xpect_eq(0);
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
			TableStore::new(store.with_subdir(RelPath::new("results")))
				.table::<Grade>();
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

	/// The builder's turn: its fill spec lands in the build directory, the
	/// only store it can write, and the build lists the filled form's cells.
	#[beet::test(timeout_ms = 120000)]
	async fn the_builder_builds_one_unit() {
		let (store, response) = Fixture::iterate(3).await;
		response.status().xpect_eq(StatusCode::OK);
		sittings(&store, "builder").await.len().xpect_eq(1);
		let fixture = Fixture::over(store.clone());
		fixture
			.read("dist/acme_course/01-business-plan.fill.json")
			.await
			.xpect_contains("Acme Stalls");
		store
			.exists(&RelPath::new("acme_course/01-business-plan.fill.json"))
			.await
			.unwrap()
			.xpect_false();
		fixture
			.read("dist/acme_course/01-business-plan.cells.md")
			.await
			.xpect_contains("Acme Stalls");
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
		let clerk = sittings(&entry.store, "clerk").await;
		clerk.len().xpect_eq(1);
		clerk[0]
			.iter()
			.filter(|text| *text == "Per weekend, as written.")
			.count()
			.xpect_eq(2);
		// a run by hand names no decision
		Fixture::threads(&entry.store, "clerk").await[0]
			.0
			.spawned_by()
			.xpect_none();
	}

	/// The loop's run is a session of one thread, the workspace's journal:
	/// each turn's decision a post of it, and each unit thread spawned by
	/// its decision, a second run continuing the journal.
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
			.filter(|post| matches!(post.as_agent_post(), Some(AgentPost::Text(_))))
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
		for (role, decision) in
			["clerk", "grader", "builder"].iter().zip(&decisions)
		{
			let units = Fixture::threads(&entry.store, role).await;
			units.len().xpect_eq(1);
			units[0]
				.0
				.spawned_by()
				.xpect_eq(Some(&PostRef::new(loop_thread.id(), decision.id())));
		}
		// a second run is another session of the same journal
		entry.request(Request::get("iterate"));
		let journal = Fixture::threads(&entry.store, "loop").await;
		journal.len().xpect_eq(1);
		journal[0]
			.1
			.iter()
			.filter_map(|post| post.log::<SessionOpened>())
			.count()
			.xpect_eq(2);
	}
}
