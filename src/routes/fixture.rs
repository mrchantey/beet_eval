//! The synthetic acme workspace behind a router serving every verb, for the
//! verbs' tests.
use crate::prelude::*;
use beet::prelude::*;

/// The fixture at `tests/fixtures/acme` copied into a store of its own, its
/// blank Word form generated through beet's `ooxml`, beneath a cli server
/// and a router serving the verbs, the check kinds, the store's cells and
/// the store view, as the fixture's `eval.bsx` mounts them, so a request is
/// refused as strictly as a command line is.
pub(crate) struct Fixture {
	world: World,
	router: Entity,
	/// The workspace store, the copy every verb reads and writes.
	pub store: BlobStore,
}

impl Fixture {
	/// The blank form's path in the reader package.
	pub const FORM: &str =
		"packages/acme_course/assets/forms/01-business-plan.docx";

	/// The fixture, freshly copied.
	pub async fn new() -> Self {
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
		world.spawn((
			ChildOf(router),
			PathPartial::new("document"),
			children![DocumentCells],
		));
		world.spawn((ChildOf(router), DocumentView));
		Self {
			world,
			router,
			store,
		}
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

	/// The blank business plan form: a name cell, a description box sharing
	/// its cell with its prompt and a red instruction sentence, and a Surveys
	/// checkbox.
	pub fn blank_form() -> MediaBytes {
		OoxmlFile::word(
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
		.unwrap()
	}
}
