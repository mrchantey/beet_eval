use crate::prelude::*;
use beet::prelude::*;

/// A workspace's manifest, `workspace.json` at the root of the workspace store:
/// where the subject and its outputs live, and the suites it is evaluated
/// against. The workspace store is the one IO goes through, resolved from an
/// ancestor; its own suite sits inside it, the rest anywhere a store uri
/// reaches.
///
/// ```json
/// {
///   "docs": "docs",
///   "results": "results",
///   "dist": "dist",
///   "suites": [
///     { "Local": "suites/acme" },
///     { "Store": { "Fs": { "path_prefix": "../beet_eval/suites/beet_biz" } } }
///   ]
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct Workspace {
	/// The documents under evaluation, laid out by the document suite's
	/// outline.
	pub docs: RelPath,
	/// What runs write and keep: the grades, the last [`Results`], the
	/// projections.
	///
	/// [`Results`]: crate::prelude::Results
	pub results: RelPath,
	/// What builds write, rebuilt at will and never kept.
	pub dist: RelPath,
	/// Every suite the subject is evaluated against, exactly one of them a
	/// document suite.
	pub suites: Vec<SuiteSource>,
}

impl Workspace {
	/// The manifest's path at the workspace store's root.
	pub const FILE: &str = "workspace.json";

	/// Reads the manifest at the root of `store`.
	pub async fn read(store: &BlobStore) -> Result<Self> {
		stored_ext::read(store, &RelPath::new(Self::FILE)).await
	}
}

impl Default for Workspace {
	fn default() -> Self {
		Self {
			docs: RelPath::new("docs"),
			results: RelPath::new("results"),
			dist: RelPath::new("dist"),
			suites: Vec::new(),
		}
	}
}

/// Where a suite lives, relative to the workspace that names it.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub enum SuiteSource {
	/// A directory of the workspace store, ie the workspace's own suite.
	Local(RelPath),
	/// Any other store; a relative `fs:` path resolves against the context
	/// directory, which a workspace run from its own root makes the
	/// workspace's.
	Store(StoreUri),
}

/// A workspace read for a run: its store, its manifest, and every suite it
/// names with its rows, the evals of all of them in one list. Read once per
/// verb; what could not be read is listed rather than raised, so `eval/check`
/// reports all of it and every other verb refuses to run on it.
#[derive(Debug, Clone)]
pub struct LoadedWorkspace {
	/// The workspace store, the one IO goes through.
	pub store: BlobStore,
	/// The manifest.
	pub manifest: Workspace,
	/// Every suite, in the manifest's order.
	pub suites: Vec<LoadedSuite>,
	/// Every eval of every suite, the outline's generated ones included, in
	/// suite order and then by id; a duplicate id keeps its first.
	pub evals: Vec<SuiteEval>,
	/// What could not be read, one line each.
	pub problems: Vec<String>,
}

/// An eval with the suite that defines it.
#[derive(Debug, Clone, PartialEq)]
pub struct SuiteEval {
	/// The defining suite's name.
	pub suite: SmolStr,
	/// The eval.
	pub eval: Eval,
	/// Whether the suite's outline generates it rather than a row.
	pub generated: bool,
}

impl LoadedWorkspace {
	/// Reads the workspace whose manifest is at the root of `store`.
	pub async fn load(store: BlobStore) -> Result<Self> {
		let manifest_path = RelPath::new(Workspace::FILE);
		if !store.exists(&manifest_path).await.unwrap_or(false) {
			bevybail!(
				"no `{}` at the root of the workspace store {}: a workspace \
				 names its documents, results and suites there",
				Workspace::FILE,
				store.describe()
			);
		}
		let manifest = Workspace::read(&store).await?;
		let mut suites = Vec::new();
		let mut problems = Vec::new();
		for source in &manifest.suites {
			let suite_store = match source {
				SuiteSource::Local(path) => store.with_subdir(path.clone()),
				SuiteSource::Store(uri) => match BlobStore::from_uri(uri) {
					Ok(suite_store) => suite_store,
					Err(err) => {
						problems.push(format!(
							"suite {source:?} does not resolve: {err}"
						));
						continue;
					}
				},
			};
			match LoadedSuite::load(suite_store, source.clone()).await {
				Ok((suite, suite_problems)) => {
					problems.extend(suite_problems);
					suites.push(suite);
				}
				Err(err) => problems.push(format!("suite {source:?}: {err}")),
			}
		}
		let mut evals = Vec::<SuiteEval>::new();
		for suite in &suites {
			let generated = match &suite.outline {
				Some(outline) => match outline.evals() {
					Ok(generated) => generated,
					Err(err) => {
						problems.push(format!(
							"{}: the outline: {err}",
							suite.manifest.name
						));
						Vec::new()
					}
				},
				None => Vec::new(),
			};
			let rows = suite.evals.iter().cloned().map(|eval| (eval, false));
			let mut own = rows
				.chain(generated.into_iter().map(|eval| (eval, true)))
				.collect::<Vec<_>>();
			own.sort_by(|(left, _), (right, _)| left.id.cmp(&right.id));
			for (eval, generated) in own {
				match evals.iter().find(|existing| existing.eval.id == eval.id)
				{
					Some(existing) => problems.push(format!(
						"{}: duplicate id {}, also defined in {}",
						suite.manifest.name, eval.id, existing.suite
					)),
					None => evals.push(SuiteEval {
						suite: suite.manifest.name.clone(),
						eval,
						generated,
					}),
				}
			}
		}
		Self {
			store,
			manifest,
			suites,
			evals,
			problems,
		}
		.xok()
	}

	/// The workspace store a verb runs against: the nearest store above its
	/// route, which an entry declares, or `--package` selects.
	pub async fn store_of(caller: &AsyncEntity) -> Result<BlobStore> {
		caller
			.with_state::<AncestorQuery<&BlobStore>, _>(|entity, query| {
				query.get(entity).cloned()
			})
			.await?
			.map_err(|_| {
				bevyhow!("no store above this verb to read a workspace from")
			})
	}

	/// Reads the workspace the verb on `caller` runs against.
	pub async fn of(caller: &AsyncEntity) -> Result<Self> {
		Self::load(Self::store_of(caller).await?).await
	}

	/// Refuses a workspace that could not be read whole, listing why.
	pub fn require_clean(&self) -> Result<&Self> {
		match self.problems.is_empty() {
			true => self.xok(),
			false => bevybail!(
				"{}\n{} problem(s): fix them before running (eval/check)",
				self.problems.join("\n"),
				self.problems.len()
			),
		}
	}

	/// The documents directory.
	pub fn docs(&self) -> BlobStore {
		self.store.with_subdir(self.manifest.docs.clone())
	}

	/// The results directory.
	pub fn results(&self) -> BlobStore {
		self.store.with_subdir(self.manifest.results.clone())
	}

	/// The build directory.
	pub fn dist(&self) -> BlobStore {
		self.store.with_subdir(self.manifest.dist.clone())
	}

	/// The suite named `name`.
	pub fn suite(&self, name: &str) -> Option<&LoadedSuite> {
		self.suites.iter().find(|suite| suite.manifest.name == name)
	}

	/// The one document suite, refusing none or several.
	pub fn document_suite(&self) -> Result<&LoadedSuite> {
		let mut documents = self
			.suites
			.iter()
			.filter(|suite| suite.manifest.kind == SuiteKind::Document);
		match (documents.next(), documents.next()) {
			(Some(suite), None) => suite.xok(),
			(None, _) => bevybail!("the workspace names no document suite"),
			(Some(first), Some(second)) => bevybail!(
				"the workspace names more than one document suite: {} and {}",
				first.manifest.name,
				second.manifest.name
			),
		}
	}

	/// The claims register: every claim of every workspace suite, in
	/// suite order.
	pub fn claims(&self) -> Vec<&Claim> {
		self.suites
			.iter()
			.filter(|suite| suite.manifest.kind == SuiteKind::Workspace)
			.flat_map(|suite| suite.claims.iter())
			.collect()
	}

	/// The document suite's outline.
	pub fn outline(&self) -> Result<&Outline> {
		let suite = self.document_suite()?;
		suite.outline.as_ref().ok_or_else(|| {
			bevyhow!(
				"the document suite {} has no outline",
				suite.manifest.name
			)
		})
	}

	/// The judged evals' ids, in suite order and then by id.
	pub fn judged(&self) -> Vec<EvalId> {
		self.evals
			.iter()
			.filter(|suite_eval| !suite_eval.eval.is_checked())
			.map(|suite_eval| suite_eval.eval.id.clone())
			.collect()
	}

	/// The eval with `id`.
	pub fn eval(&self, id: &EvalId) -> Option<&SuiteEval> {
		self.evals
			.iter()
			.find(|suite_eval| &suite_eval.eval.id == id)
	}

	/// Every rubric of every suite, `<suite>/<rubric>`, in that order.
	pub fn rubrics(&self) -> Vec<(RubricRef, &Rubric)> {
		let mut rubrics = self
			.suites
			.iter()
			.flat_map(|suite| {
				suite.rubrics.iter().filter_map(move |rubric| {
					RubricRef::new(&suite.manifest.name, &rubric.id)
						.ok()
						.map(|reference| (reference, rubric))
				})
			})
			.collect::<Vec<_>>();
		rubrics.sort_by(|(left, _), (right, _)| left.cmp(right));
		rubrics
	}

	/// The rubric `reference` names.
	pub fn rubric(&self, reference: &RubricRef) -> Option<&Rubric> {
		self.suite(reference.suite())?
			.rubrics
			.iter()
			.find(|rubric| rubric.id == reference.rubric())
	}
}
