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
	/// template.
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
