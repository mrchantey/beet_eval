//! The synthetic acme workspace behind a router serving every verb, for the
//! verbs' tests.
use crate::prelude::*;
use beet::prelude::*;
use std::sync::Arc;
use std::sync::Mutex;

/// The fixture at `tests/fixtures/acme` copied into a store of its own, its
/// blank Word form generated through beet's `WordFixture`, beneath a cli
/// server and a router serving the verbs, the check kinds and the store
/// view, as the fixture's `eval.bsx` mounts them, so a request is refused as
/// strictly as a command line is.
pub(crate) struct Fixture {
	world: World,
	router: Entity,
	/// The workspace store, the copy every verb reads and writes.
	pub store: BlobStore,
}

impl Fixture {
	/// The blank form's path in the reader suite.
	pub const FORM: &str =
		"suites/acme_course/assets/forms/01-business-plan.docx";

	/// The business plan's answer sheet in the workspace store.
	pub const ANSWERS: &str = "dist/acme_course/01-business-plan.answers.md";

	/// The fixture, freshly copied.
	pub async fn new() -> Self { Self::over(Self::workspace().await) }

	/// The verbs over `store`, ie a copy a loop has run in.
	pub fn over(store: BlobStore) -> Self {
		let mut world = (AsyncPlugin, RouterPlugin, EvalPlugin).into_world();
		let server = world.spawn((CliServer::default(), store.clone())).id();
		let router = world
			.spawn((
				ChildOf(server),
				Router,
				RequestLogger::default(),
				HelpHandler::default(),
			))
			.id();
		world
			.spawn((ChildOf(router), PathPartial::new("eval")))
			.insert_template(EvalRoutes)
			.unwrap();
		world
			.spawn((ChildOf(router), PathPartial::new("check")))
			.insert_template(CheckRoutes)
			.unwrap();
		world.spawn((ChildOf(router), DocumentView));
		Self {
			world,
			router,
			store,
		}
	}

	/// A fresh copy of the fixture's workspace, its blank Word form
	/// generated.
	pub async fn workspace() -> BlobStore {
		let source = BlobStore::new(FsStore::new(
			AbsPath::new_manifest_rel("tests/fixtures/acme").unwrap(),
		));
		let store = BlobStore::temp();
		for path in source.list().await.unwrap() {
			store
				.insert(&path, source.get(&path).await.unwrap())
				.await
				.unwrap();
		}
		store
			.insert(
				&RelPath::new(Self::FORM),
				Self::blank_form().bytes().to_vec(),
			)
			.await
			.unwrap();
		store
	}

	/// `turns` turns of the fixture's loop, its `iterate` route requested of
	/// a fresh [`FixtureEntry`]: the copy as the roles left it, and the run's
	/// answer.
	pub async fn iterate(turns: u32) -> (BlobStore, Response) {
		let mut entry = FixtureEntry::looping(turns).await;
		let response = entry.request(Request::get("iterate"));
		(entry.store, response)
	}

	/// The request a command line makes, ie
	/// `eval/next --accept=application/json`: its `--accept` is the request's
	/// `Accept`, as the cli server makes it, markdown when unset, as an agent
	/// reads.
	pub fn request(command: &str) -> Request {
		let request = Request::from_cli_str(command);
		let accept = request
			.get_param("accept")
			.map(|accept| MediaType::from_accepts(accept))
			.unwrap_or_else(|| vec![MediaType::Markdown]);
		request.with_header::<header::Accept>(accept)
	}

	/// The answer to a command line: whether it succeeded, and its text.
	pub async fn call(&mut self, command: &str) -> (bool, String) {
		self.answer(Self::request(command)).await
	}

	/// The answer to `request`, for params a command line cannot spell.
	pub async fn answer(&mut self, request: Request) -> (bool, String) {
		let response = self.exchange(request).await;
		let ok = response.status().is_ok();
		(ok, response.text().await.unwrap())
	}

	/// The response to `request`.
	pub async fn exchange(&mut self, request: Request) -> Response {
		self.world.entity_mut(self.router).exchange(request).await
	}

	/// The answer to a command line that must succeed.
	pub async fn ok(&mut self, command: &str) -> String {
		match self.call(command).await {
			(true, text) => text,
			(false, text) => panic!("`{command}` failed: {text}"),
		}
	}

	/// The answer to a command line that must fail.
	pub async fn refused(&mut self, command: &str) -> String {
		match self.call(command).await {
			(false, text) => text,
			(true, text) => panic!("`{command}` succeeded: {text}"),
		}
	}

	/// Every route the router serves.
	pub fn routes(&self) -> Vec<ActionNode> {
		self.tree().flatten_nodes().into_iter().cloned().collect()
	}

	/// The route `request` reaches.
	pub fn route_of(&self, request: &Request) -> Option<ActionNode> {
		self.tree().find(request.path()).cloned()
	}

	fn tree(&self) -> &RouteTree {
		RouteTree::of(&self.world, self.router).unwrap()
	}

	/// The text at `path` in the workspace store.
	pub async fn read(&self, path: &str) -> String {
		String::from_utf8(
			self.store.get(&RelPath::new(path)).await.unwrap().to_vec(),
		)
		.unwrap()
	}

	/// Writes `sheet` as the business plan's answers and approves it.
	pub async fn approved_answers(&mut self, sheet: &str) {
		self.write(Self::ANSWERS, sheet).await;
		self.ok("eval/approve acme_course/01-business-plan --by=ada")
			.await;
	}

	/// Writes `text` at `path` in the workspace store.
	pub async fn write(&self, path: &str, text: &str) {
		self.store
			.insert(&RelPath::new(path), text.to_string())
			.await
			.unwrap();
	}

	/// Every thread listed under `dir` of the workspace suite's threads in
	/// `store`, ie `clerk/product` for the clerk's threads about the product,
	/// with its posts in order, oldest thread first.
	pub async fn threads(
		store: &BlobStore,
		dir: &str,
	) -> Vec<(Thread, Vec<Post>)> {
		let records = ThreadStore::new(
			JsonTableStore::over(
				store.with_subdir(RelPath::new("suites/acme/assets/threads")),
			)
			.unwrap(),
			RelPath::new(dir),
		)
		.unwrap();
		let mut threads = records.threads().await.unwrap_or_default();
		threads.sort_by_key(|thread| (thread.created(), thread.id()));
		let mut out = Vec::new();
		for thread in threads {
			let posts = records.posts(thread.id()).await.unwrap();
			out.push((thread, posts));
		}
		out
	}

	/// The texts of every thread listed under `dir` in `store`, each its
	/// posts in order, a function call as `call <tool path>`, its output as
	/// `output <text>`, the log posts left out.
	pub async fn sittings(store: &BlobStore, dir: &str) -> Vec<Vec<String>> {
		Self::threads(store, dir)
			.await
			.into_iter()
			.map(|(_, posts)| {
				posts
					.iter()
					.filter_map(|post| match post.as_agent_post()? {
						AgentPost::FunctionCall(call) => Some(format!(
							"call {}",
							FunctionToolDefinition::decode_name(call.name())
						)),
						AgentPost::FunctionCallOutput(output) => {
							Some(format!("output {}", output.output()))
						}
						_ => Some(post.to_string()),
					})
					.collect()
			})
			.collect()
	}

	/// The blank business plan form: a name cell, a description box sharing
	/// its cell with its prompt and a red instruction sentence, and a Surveys
	/// checkbox.
	pub fn blank_form() -> MediaBytes {
		WordFixture::new(
			"<w:tbl><w:tr>\
			 <w:tc><w:p><w:r><w:t>Business Name</w:t></w:r></w:p></w:tc>\
			 <w:tc><w:p/></w:tc>\
			 </w:tr></w:tbl>\
			 <w:tbl><w:tr><w:tc>\
			 <w:p><w:r><w:t>Briefly describe your business:</w:t></w:r></w:p>\
			 <w:p><w:r><w:rPr><w:color w:val=\"FF0000\"/></w:rPr><w:t>(Please delete this sentence once completed)</w:t></w:r></w:p>\
			 </w:tc></w:tr></w:tbl>\
			 <w:p><w:sdt><w:sdtPr><w14:checkbox><w14:checked w14:val=\"0\"/></w14:checkbox></w:sdtPr>\
			 <w:sdtContent><w:r><w:t>\u{2610}</w:t></w:r></w:sdtContent></w:sdt>\
			 <w:r><w:t xml:space=\"preserve\"> Surveys</w:t></w:r></w:p>",
		)
		.bytes()
	}

	/// `func` run on `source` read into a world of the formats.
	pub fn read_with<O>(
		source: &MediaBytes,
		func: impl FnOnce(&mut World, Entity) -> O,
	) -> O {
		let mut world =
			(TemplatePlugin, DocumentPlugin, OoxmlPlugin).into_world();
		let root = world.spawn_empty().id();
		ParseContext::new(&mut world.entity_mut(root), source)
			.parse()
			.unwrap();
		func(&mut world, root)
	}

	/// The cells of `form`, as a build lists them.
	pub fn cells_of(form: &MediaBytes) -> Vec<FormCell> {
		Self::read_with(form, FormCell::listing)
	}

	/// `source` read and written as `media_type`, ie an HTML scene of cards
	/// as a deck.
	pub fn written(source: &MediaBytes, media_type: MediaType) -> MediaBytes {
		Self::read_with(source, |world, root| {
			DocRenderers::render(
				world,
				root,
				&RequestParts::get("/").with_accept(media_type),
			)
			.unwrap()
		})
	}
}

/// One of the fixture's entries built as the binary builds it, over a fresh
/// copy of the workspace, every agent scripted: `iterate.bsx`, the loop at
/// `iterate` and each role at its own route, ie `grade/product`, or
/// `coach.bsx`, the coach's sitting held by its `RunThread`, started by
/// [`Self::start`].
pub(crate) struct FixtureEntry {
	app: App,
	router: Entity,
	/// The workspace store, the copy every role reads and writes.
	pub store: BlobStore,
}

impl FixtureEntry {
	/// `iterate.bsx` over a fresh copy of the workspace, built and disarmed.
	pub async fn new() -> Self { Self::of("iterate.bsx").await }

	/// `iterate.bsx` whose loop takes `turns` turns a run, in place of its
	/// own count, over a fresh copy of the workspace.
	pub async fn looping(turns: u32) -> Self {
		let store = Fixture::workspace().await;
		let path = RelPath::new(Self::LOOP_THREAD);
		let source =
			String::from_utf8(store.get(&path).await.unwrap().to_vec())
				.unwrap();
		let count = "total_times=3";
		source.contains(count).xpect_true();
		store
			.insert(
				&path,
				source.replace(count, &format!("total_times={turns}")),
			)
			.await
			.unwrap();
		Self::build(store, "iterate.bsx").await
	}

	/// `iterate.bsx` over `store`, ie a copy a loop has run in.
	pub async fn over(store: BlobStore) -> Self {
		Self::build(store, "iterate.bsx").await
	}

	/// The entry `name` over a fresh copy of the workspace, built and
	/// disarmed.
	pub async fn of(name: &str) -> Self {
		Self::build(Fixture::workspace().await, name).await
	}

	/// The template of the loop's thread, its turns a run its one count.
	const LOOP_THREAD: &str = "templates/LoopThread.bsx";

	/// The entry `name` over `store`, built as the binary builds it and
	/// disarmed.
	async fn build(store: BlobStore, name: &str) -> Self {
		let mut app = App::new();
		app.add_plugins(MinimalPlugins)
			.install_types::<(RouterPlugin, EvalPlugin)>();
		let prescans = app
			.world_mut()
			.get_resource_or_init::<PrescanRegistry>()
			.clone();
		let entry = entry_build::resolve_in_package_store(
			store.clone(),
			&prescans,
			name.into(),
		)
		.await
		.unwrap();
		let sources = entry_build::read_sources(
			&entry.package_store,
			entry.entry_name,
			entry.prescan,
		)
		.await
		.unwrap();
		let root = entry_build::build_root(
			app.world_mut(),
			entry.package_store,
			sources,
			DisableCallOnReady,
		)
		.unwrap();
		let router = app
			.world_mut()
			.run_system_cached_with::<_, Result<Entity>, _, _>(
				find_router,
				root,
			)
			.unwrap()
			.unwrap();
		Self { app, router, store }
	}

	/// The response to `request`, the app updated until it answers.
	pub fn request(&mut self, request: Request) -> Response {
		self.until(self.router, async move |router| {
			router.exchange(request).await
		})
	}

	/// The outcome of starting the entry with `request`, as the start verb
	/// starts it, ie `Request::get("/").with_param("claim", "per-weekend")`:
	/// its one `RunThread` called with the request, the app updated until
	/// the run ends.
	pub fn start(&mut self, request: Request) -> Result<Outcome> {
		let run = self
			.app
			.world_mut()
			.query_filtered::<Entity, With<RunThread>>()
			.single(self.app.world())
			.unwrap();
		self.until(run, async move |run| {
			run.call::<Request, Outcome>(request).await
		})
	}

	/// What `func` answers of `entity`, the app updated until it does.
	fn until<O: 'static + Send>(
		&mut self,
		entity: Entity,
		func: impl 'static + AsyncFnOnce(AsyncEntity) -> O,
	) -> O {
		let answer = Arc::new(Mutex::new(None));
		let out = answer.clone();
		self.app.world_mut().entity_mut(entity).run_async_local(
			|entity| async move {
				*out.lock().unwrap() = Some(func(entity).await);
				Ok(())
			},
		);
		for _ in 0..20_000 {
			self.app.update();
			if let Some(answer) = answer.lock().unwrap().take() {
				return answer;
			}
		}
		panic!("the entry did not answer");
	}
}
