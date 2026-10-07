use crate::prelude::*;
use beet::prelude::*;

/// Every document under a workspace's documents directory, each file read
/// once: what the check kinds, the triage and the briefs read the subject
/// through. A document is named by its path without the extension, a
/// promoted one by its directory, so `market` is `market.md` or
/// `market/index.md` and `market/customers` its child.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DocumentSet {
	/// The documents directory as the workspace names it, ie `docs`, which
	/// a message prefixes a file with.
	dir: RelPath,
	/// Every file read, in path order, as its document.
	files: Vec<MarkdownDocument>,
}

impl DocumentSet {
	/// Reads every markdown file of `store`, the documents directory the
	/// workspace names `dir`; an absent directory is an empty set.
	pub async fn load(store: &BlobStore, dir: RelPath) -> Result<Self> {
		if !store.store_exists().await.unwrap_or(false) {
			return Self::from_sources(dir, Vec::new());
		}
		let mut sources = Vec::new();
		for path in store.list().await? {
			if path.as_str().ends_with(".md") {
				let bytes = store.get(&path).await?;
				sources.push((path, String::from_utf8(bytes.to_vec())?));
			}
		}
		Self::from_sources(dir, sources)
	}

	/// The set of `sources`, each a file's path under the documents
	/// directory `dir` and its text.
	pub fn from_sources(
		dir: RelPath,
		mut sources: Vec<(RelPath, String)>,
	) -> Result<Self> {
		sources
			.sort_by(|(left, _), (right, _)| left.as_str().cmp(right.as_str()));
		let mut files = sources
			.into_iter()
			.map(|(path, text)| {
				MarkdownDocument::parse(Self::name_of(&path), path, &text)
			})
			.collect::<Result<Vec<_>>>()?;
		// a promoted document holds its children, read through it
		let snapshot = files.clone();
		for file in &mut files {
			if file.file.as_str().ends_with("/index.md") {
				file.children = snapshot
					.iter()
					.filter(|other| Self::is_child(&file.name, &other.name))
					.cloned()
					.collect();
			}
		}
		Self { dir, files }.xok()
	}

	/// The documents directory as the workspace names it.
	pub fn dir(&self) -> &RelPath { &self.dir }

	/// A document's file as a message names it, ie `docs/brand.md`.
	pub fn path_of(&self, document: &MarkdownDocument) -> String {
		format!("{}/{}", self.dir, document.file)
	}

	/// A document's address as a message names it, ie `docs/brand`.
	pub fn path_name(&self, name: &str) -> String {
		format!("{}/{name}", self.dir)
	}

	/// The document named `name`, `<name>.md` before `<name>/index.md`.
	pub fn get(&self, name: &str) -> Option<&MarkdownDocument> {
		let file = format!("{name}.md");
		let index = format!("{name}/index.md");
		self.files
			.iter()
			.find(|document| document.file.as_str() == file)
			.or_else(|| {
				self.files
					.iter()
					.find(|document| document.file.as_str() == index)
			})
	}

	/// Every file's document, in path order.
	pub fn files(&self) -> &[MarkdownDocument] { &self.files }

	/// Whether no document exists.
	pub fn is_empty(&self) -> bool { self.files.is_empty() }

	/// The top level document a file belongs to, ie `market` for
	/// `market/customers.md`.
	pub fn top_level(document: &MarkdownDocument) -> &str {
		document
			.name
			.split_once('/')
			.map_or(document.name.as_str(), |(top, _)| top)
	}

	/// Every named block, in file then line order, with the document it is
	/// in.
	pub fn blocks(&self) -> Vec<(&MarkdownDocument, &DataBlock)> {
		self.files
			.iter()
			.flat_map(|document| {
				document.blocks.iter().map(move |block| (document, block))
			})
			.collect()
	}

	/// Every ask, in file then line order, with the document it is in.
	pub fn asks(&self) -> Vec<(&MarkdownDocument, &Ask)> {
		self.files
			.iter()
			.flat_map(|document| {
				document.asks.iter().map(move |ask| (document, ask))
			})
			.collect()
	}

	/// The text at `address`: a section's body, or a whole document's summary.
	pub fn text_at(&self, address: &Address) -> Option<&str> {
		let document = self.get(address.document_name())?;
		match address.section() {
			Some(slug) => {
				document.section(slug).map(|section| section.body.as_str())
			}
			None => document.summary.as_deref(),
		}
	}

	/// A file's document name: its path without `.md`, an `index.md` naming
	/// its directory.
	fn name_of(path: &RelPath) -> SmolStr {
		let path = path.as_str();
		let stem = path.strip_suffix(".md").unwrap_or(path);
		stem.strip_suffix("/index").unwrap_or(stem).into()
	}

	/// Whether `child` lies under the promoted document `parent`.
	fn is_child(parent: &str, child: &str) -> bool {
		child
			.strip_prefix(parent)
			.is_some_and(|rest| rest.starts_with('/'))
	}
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	#[beet::test]
	fn resolves_promoted_documents() {
		let set = DocumentSet::from_sources(RelPath::new("docs"), vec![
			(
				RelPath::new("market/index.md"),
				"# Market\n\n## Research\n".into(),
			),
			(RelPath::new("market/customers.md"), "# Customers\n".into()),
			(RelPath::new("brand.md"), "# Brand\n".into()),
		])
		.unwrap();
		set.files()
			.iter()
			.map(|document| document.name.as_str())
			.collect::<Vec<_>>()
			.xpect_eq(vec!["brand", "market/customers", "market"]);
		let market = set.get("market").unwrap();
		market.file.as_str().xpect_eq("market/index.md");
		market.children[0]
			.name
			.as_str()
			.xpect_eq("market/customers");
		set.get("market/customers")
			.unwrap()
			.title
			.clone()
			.unwrap()
			.xpect_eq("Customers");
		DocumentSet::top_level(set.get("market/customers").unwrap())
			.xpect_eq("market");
		set.get("missing").xpect_none();
	}
}
