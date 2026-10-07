use crate::prelude::*;
use beet::prelude::*;

/// A workspace's manifest, `workspace.json` at the root of the workspace store:
/// where the subject and its outputs live, and the packages it is evaluated
/// against. The workspace store is the one IO goes through, resolved from an
/// ancestor; its own package sits inside it, the rest anywhere a store uri
/// reaches.
///
/// ```json
/// {
///   "docs": "docs",
///   "results": "results",
///   "dist": "dist",
///   "packages": [
///     { "Local": "packages/acme" },
///     { "Store": { "Fs": { "path_prefix": "../beet_eval/packages/beet_biz" } } }
///   ]
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct Workspace {
	/// The documents under evaluation, laid out by the document package's
	/// outline.
	pub docs: RelPath,
	/// What runs write and keep: the grades, the last [`Results`], the
	/// projections.
	///
	/// [`Results`]: crate::prelude::Results
	pub results: RelPath,
	/// What builds write, rebuilt at will and never kept.
	pub dist: RelPath,
	/// Every package the subject is evaluated against, exactly one of them a
	/// document package.
	pub packages: Vec<PackageSource>,
}

impl Workspace {
	/// The manifest's path at the workspace store's root.
	pub const FILE: &str = "workspace.json";

	/// Reads the manifest at the root of `store`.
	pub async fn read(store: &BlobStore) -> Result<Self> {
		json_ext::read(store, &RelPath::new(Self::FILE)).await
	}
}

impl Default for Workspace {
	fn default() -> Self {
		Self {
			docs: RelPath::new("docs"),
			results: RelPath::new("results"),
			dist: RelPath::new("dist"),
			packages: Vec::new(),
		}
	}
}

/// Where a package lives, relative to the workspace that names it.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub enum PackageSource {
	/// A directory of the workspace store, ie the workspace's own package.
	Local(RelPath),
	/// Any other store; a relative `fs:` path resolves against the context
	/// directory, which a workspace run from its own root makes the
	/// workspace's.
	Store(StoreUri),
}

/// A workspace read for a run: its store, its manifest, and every package it
/// names with its rows, the evals of all of them in one list. Read once per
/// verb; what could not be read is listed rather than raised, so `eval/check`
/// reports all of it and every other verb refuses to run on it.
#[derive(Debug, Clone)]
pub struct LoadedWorkspace {
	/// The workspace store, the one IO goes through.
	pub store: BlobStore,
	/// The manifest.
	pub manifest: Workspace,
	/// Every package, in the manifest's order.
	pub packages: Vec<LoadedPackage>,
	/// Every eval of every package, the outline's generated ones included, in
	/// package order and then by id; a duplicate id keeps its first.
	pub evals: Vec<PackagedEval>,
	/// What could not be read, one line each.
	pub problems: Vec<String>,
}

/// An eval with the package that defines it.
#[derive(Debug, Clone, PartialEq)]
pub struct PackagedEval {
	/// The defining package's name.
	pub package: SmolStr,
	/// The eval.
	pub eval: Eval,
	/// Whether the package's outline generates it rather than a row.
	pub generated: bool,
}

impl LoadedWorkspace {
	/// Reads the workspace whose manifest is at the root of `store`.
	pub async fn load(store: BlobStore) -> Result<Self> {
		let manifest_path = RelPath::new(Workspace::FILE);
		if !store.exists(&manifest_path).await.unwrap_or(false) {
			bevybail!(
				"no `{}` at the root of the workspace store {}: a workspace \
				 names its documents, results and packages there",
				Workspace::FILE,
				store.describe()
			);
		}
		let manifest = Workspace::read(&store).await?;
		let mut packages = Vec::new();
		let mut problems = Vec::new();
		for source in &manifest.packages {
			let package_store = match source {
				PackageSource::Local(path) => store.with_subdir(path.clone()),
				PackageSource::Store(uri) => match BlobStore::from_uri(uri) {
					Ok(package_store) => package_store,
					Err(err) => {
						problems.push(format!(
							"package {source:?} does not resolve: {err}"
						));
						continue;
					}
				},
			};
			match LoadedPackage::load(package_store, source.clone()).await {
				Ok((package, package_problems)) => {
					problems.extend(package_problems);
					packages.push(package);
				}
				Err(err) => problems.push(format!("package {source:?}: {err}")),
			}
		}
		let mut evals = Vec::<PackagedEval>::new();
		for package in &packages {
			let generated = match &package.outline {
				Some(outline) => match outline.evals() {
					Ok(generated) => generated,
					Err(err) => {
						problems.push(format!(
							"{}: the outline: {err}",
							package.manifest.name
						));
						Vec::new()
					}
				},
				None => Vec::new(),
			};
			let rows = package.evals.iter().cloned().map(|eval| (eval, false));
			let mut own = rows
				.chain(generated.into_iter().map(|eval| (eval, true)))
				.collect::<Vec<_>>();
			own.sort_by(|(left, _), (right, _)| left.id.cmp(&right.id));
			for (eval, generated) in own {
				match evals.iter().find(|existing| existing.eval.id == eval.id)
				{
					Some(existing) => problems.push(format!(
						"{}: duplicate id {}, also defined in {}",
						package.manifest.name, eval.id, existing.package
					)),
					None => evals.push(PackagedEval {
						package: package.manifest.name.clone(),
						eval,
						generated,
					}),
				}
			}
		}
		Self {
			store,
			manifest,
			packages,
			evals,
			problems,
		}
		.xok()
	}

	/// The workspace store a verb runs against: the nearest store above its
	/// route, which an entry declares, or `--repo` selects.
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

	/// The package named `name`.
	pub fn package(&self, name: &str) -> Option<&LoadedPackage> {
		self.packages
			.iter()
			.find(|package| package.manifest.name == name)
	}

	/// The one document package, refusing none or several.
	pub fn document_package(&self) -> Result<&LoadedPackage> {
		let mut documents = self
			.packages
			.iter()
			.filter(|package| package.manifest.kind == PackageKind::Document);
		match (documents.next(), documents.next()) {
			(Some(package), None) => package.xok(),
			(None, _) => bevybail!("the workspace names no document package"),
			(Some(first), Some(second)) => bevybail!(
				"the workspace names more than one document package: {} and {}",
				first.manifest.name,
				second.manifest.name
			),
		}
	}

	/// The document package's outline.
	pub fn outline(&self) -> Result<&Outline> {
		let package = self.document_package()?;
		package.outline.as_ref().ok_or_else(|| {
			bevyhow!(
				"the document package {} has no outline",
				package.manifest.name
			)
		})
	}

	/// The eval with `id`.
	pub fn eval(&self, id: &EvalId) -> Option<&PackagedEval> {
		self.evals.iter().find(|packaged| &packaged.eval.id == id)
	}

	/// Every rubric of every package, `<package>/<rubric>`, in that order.
	pub fn rubrics(&self) -> Vec<(RubricRef, &Rubric)> {
		let mut rubrics = self
			.packages
			.iter()
			.flat_map(|package| {
				package.rubrics.iter().filter_map(move |rubric| {
					RubricRef::new(&package.manifest.name, &rubric.id)
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
		self.package(reference.package())?
			.rubrics
			.iter()
			.find(|rubric| rubric.id == reference.rubric())
	}
}
