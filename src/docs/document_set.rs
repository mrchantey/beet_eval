use crate::prelude::*;
use beet::exports::bevy::reflect::TypeRegistry;
use beet::prelude::*;

/// Every document under a workspace's documents directory, each file parsed
/// once by beet's media parse into a document root of a world the set owns:
/// what the check kinds, the triage and the briefs read the subject through,
/// by the traversals of [`ProseQuery`]. A markdown file and a Word file
/// read alike. A document is named by its path without the extension, a
/// promoted one by its directory, so `market` is `market.md` or
/// `market/index.md` and `market/customers` its child.
pub struct DocumentSet {
	/// The documents directory as the workspace names it, ie `docs`, which
	/// a message prefixes a file with.
	dir: RelPath,
	/// The world the documents are parsed into.
	world: World,
	/// Every file read, in path order.
	files: Vec<DocumentFile>,
}

/// One file of a [`DocumentSet`]: its name, its path and its root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentFile {
	/// The document's address without a section, ie `market`, or
	/// `market/customers` for a promoted child.
	pub name: SmolStr,
	/// The file, relative to the documents directory, ie `market.md`.
	pub file: RelPath,
	/// The document's root in the set's world.
	pub root: Entity,
	/// What could not be read, ie frontmatter that does not parse.
	pub problems: Vec<String>,
}

impl DocumentSet {
	/// The [`PageMeta`] keys every document's frontmatter carries, in the order
	/// a document writes them.
	pub const META_KEYS: [&str; 3] = ["created", "updated", "authors"];

	/// The extensions of the files a documents directory holds documents in.
	const EXTENSIONS: [&str; 2] = ["md", "docx"];

	/// Reads every document of `store`, the documents directory the
	/// workspace names `dir`; an absent directory is an empty set.
	pub async fn load(store: &BlobStore, dir: RelPath) -> Result<Self> {
		let mut sources = Vec::new();
		if store.store_exists().await.unwrap_or(false) {
			for path in store.list().await? {
				if Self::EXTENSIONS
					.contains(&path.extension().unwrap_or_default())
				{
					let bytes = store.blob(path.clone()).get_media().await?;
					sources.push((path, bytes));
				}
			}
		}
		Self::from_sources(dir, sources)
	}

	/// The set of `sources`, each a file's path under the documents
	/// directory `dir` and its bytes.
	pub fn from_sources(
		dir: RelPath,
		mut sources: Vec<(RelPath, MediaBytes)>,
	) -> Result<Self> {
		sources
			.sort_by(|(left, _), (right, _)| left.as_str().cmp(right.as_str()));
		let mut world = (TemplatePlugin, DocumentPlugin).into_world();
		let mut files = Vec::new();
		for (path, bytes) in sources {
			let root = world.spawn_empty().id();
			let mut problems = Vec::new();
			match MediaParser::scan_root_declarations(
				&bytes,
				&type_ext::short_name::<PageMeta>(),
			)
			.and_then(|declarations| {
				declarations.get::<PageMeta>(Self::meta_registry())
			}) {
				Ok(Some(meta)) => {
					world.entity_mut(root).insert(meta);
				}
				Ok(None) => {}
				Err(err) => problems
					.push(format!("the frontmatter does not parse: {err}")),
			}
			MediaParser::new()
				.parse(
					ParseContext::new(&mut world.entity_mut(root), &bytes)
						.with_path(WsPath::new(path.as_str())),
				)
				.map_err(|err| bevyhow!("{dir}/{path}: {err}"))?;
			files.push(DocumentFile {
				name: Self::name_of(&path),
				file: path,
				root,
				problems,
			});
		}
		Self { dir, world, files }.xok()
	}

	/// The registry frontmatter is read against, holding [`PageMeta`].
	fn meta_registry() -> &'static TypeRegistry {
		static REGISTRY: std::sync::OnceLock<TypeRegistry> =
			std::sync::OnceLock::new();
		REGISTRY.get_or_init(|| {
			let mut registry = TypeRegistry::default();
			registry.register::<PageMeta>();
			registry
		})
	}

	/// The documents directory as the workspace names it.
	pub fn dir(&self) -> &RelPath { &self.dir }

	/// A document's file as a message names it, ie `docs/brand.md`.
	pub fn path_of(&self, document: &DocumentFile) -> String {
		format!("{}/{}", self.dir, document.file)
	}

	/// A document's address as a message names it, ie `docs/brand`.
	pub fn path_name(&self, name: &str) -> String {
		format!("{}/{name}", self.dir)
	}

	/// The document named `name`, a file of its own before a directory's
	/// index.
	pub fn get(&self, name: &str) -> Option<&DocumentFile> {
		let own =
			Self::EXTENSIONS.map(|extension| format!("{name}.{extension}"));
		let index = Self::EXTENSIONS
			.map(|extension| format!("{name}/index.{extension}"));
		own.iter().chain(index.iter()).find_map(|path| {
			self.files
				.iter()
				.find(|document| document.file.as_str() == path)
		})
	}

	/// Every file's document, in path order.
	pub fn files(&self) -> &[DocumentFile] { &self.files }

	/// Whether no document exists.
	pub fn is_empty(&self) -> bool { self.files.is_empty() }

	/// The top level document a file belongs to, ie `market` for
	/// `market/customers.md`.
	pub fn top_level(document: &DocumentFile) -> &str {
		document
			.name
			.split_once('/')
			.map_or(document.name.as_str(), |(top, _)| top)
	}

	/// Runs `func` with the traversals over the set's world.
	pub fn query<O>(&mut self, func: impl FnOnce(&ProseQuery) -> O) -> O {
		self.world.with_state::<ProseQuery, _>(|query| func(&query))
	}

	/// Every named data block, in file then document order, with the
	/// document it is in.
	pub fn data_blocks(&mut self) -> Vec<(DocumentFile, DataBlock)> {
		let files = self.files.clone();
		self.query(|query| {
			files
				.into_iter()
				.flat_map(|document| {
					query
						.data_blocks(document.root)
						.into_iter()
						.map(move |block| (document.clone(), block))
				})
				.collect()
		})
	}

	/// Every ask, in file then document order, with the document it is in.
	pub fn asks(&mut self) -> Vec<(DocumentFile, Ask)> {
		let files = self.files.clone();
		self.query(|query| {
			files
				.into_iter()
				.flat_map(|document| {
					query
						.asks(document.root)
						.into_iter()
						.map(move |ask| (document.clone(), ask))
				})
				.collect()
		})
	}

	/// The markdown a reader reads at `address`: a section's blocks, or a
	/// whole document's summary.
	pub fn text_at(&mut self, address: &Address) -> Option<String> {
		let root = self.get(address.document_name())?.root;
		let blocks = self.query(|query| match address.section() {
			Some(slug) => {
				query.section(root, slug).map(|section| section.blocks)
			}
			None => query.summary(root).map(|summary| vec![summary]),
		})?;
		self.markdown(&blocks).xsome()
	}

	/// The markdown of everything a reader reads in a document, its head and
	/// every section: what a quote anchored at the whole document is looked
	/// for in.
	pub fn text(&mut self, document: &DocumentFile) -> String {
		let root = document.root;
		let blocks = self.query(|query| query.blocks(root));
		self.markdown(&blocks)
	}

	/// The markdown of a section of `document`.
	pub fn section_text(
		&mut self,
		document: &DocumentFile,
		slug: &str,
	) -> Option<String> {
		let root = document.root;
		let blocks = self.query(|query| {
			query.section(root, slug).map(|section| section.blocks)
		})?;
		self.markdown(&blocks).xsome()
	}

	/// `blocks` rendered as markdown, a blank line between each.
	fn markdown(&mut self, blocks: &[Entity]) -> String {
		blocks
			.iter()
			.filter_map(|block| {
				MarkdownRenderer::new()
					.render(&mut RenderContext::new(*block, &mut self.world))
					.ok()
					.map(|bytes| bytes.to_string().trim().to_string())
			})
			.filter(|text| !text.is_empty())
			.collect::<Vec<_>>()
			.join("\n\n")
	}

	/// A file's document name: its path without its extension, an `index`
	/// naming its directory.
	fn name_of(path: &RelPath) -> SmolStr {
		let path = path.as_str();
		let stem = Self::EXTENSIONS
			.iter()
			.find_map(|extension| path.strip_suffix(&format!(".{extension}")))
			.unwrap_or(path);
		stem.strip_suffix("/index").unwrap_or(stem).into()
	}
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	const BRAND: &str = "---
created: 2026-10-02
updated: 2026-10-04
authors: [Ada Lovelace]
---

# Acme Stalls

*Stalls that come fitted*

Acme rents fitted market stalls by the month.

## Name and tagline

The name is Acme Stalls. TODO(ask): the tagline's reasoning.

```csv price-list
line,price_ex_gst
\"Stall, fitted\",120
```

## Voice

TODO(ask decision: playful or plain?) and TODO(ask fact: who reads it)

```md
## not a section
```
";

	fn set(sources: Vec<(&str, MediaBytes)>) -> DocumentSet {
		DocumentSet::from_sources(
			RelPath::new("docs"),
			sources
				.into_iter()
				.map(|(path, bytes)| (RelPath::new(path), bytes))
				.collect(),
		)
		.unwrap()
	}

	#[beet::test]
	fn reads_a_document() {
		let mut set = set(vec![("brand.md", MediaBytes::new_markdown(BRAND))]);
		let root = set.get("brand").unwrap().root;
		set.query(|query| {
			let meta = query.meta(root).unwrap().clone();
			meta.created.unwrap().to_string().xpect_eq("2026-10-02");
			meta.authors.xpect_eq(vec![SmolStr::new("Ada Lovelace")]);
			query.title(root).unwrap().xpect_eq("Acme Stalls");
			query
				.tagline(root)
				.unwrap()
				.xpect_eq("Stalls that come fitted");
			query
				.words(query.summary(root).unwrap())
				.xpect_eq("Acme rents fitted market stalls by the month.");
			query
				.sections(root)
				.iter()
				.map(|section| (section.slug.to_string(), section.line))
				.collect::<Vec<_>>()
				.xpect_eq(vec![
					("name-and-tagline".to_string(), 13),
					("voice".into(), 22),
				]);
			let blocks = query.data_blocks(root);
			blocks.len().xpect_eq(1);
			(
				blocks[0].name.as_str(),
				blocks[0].line,
				blocks[0].section.clone(),
			)
				.xpect_eq((
					"price-list",
					17,
					Some(SmolStr::new("name-and-tagline")),
				));
			blocks[0].rows.clone().xpect_eq(vec![vec![
				"Stall, fitted".to_string(),
				"120".into(),
			]]);
			query
				.asks(root)
				.iter()
				.map(|ask| (ask.kind, ask.text.clone(), ask.line))
				.collect::<Vec<_>>()
				.xpect_eq(vec![
					(AskKind::Fact, "the tagline's reasoning.".to_string(), 15),
					(AskKind::Decision, "playful or plain?".into(), 24),
					(AskKind::Fact, "who reads it".into(), 24),
				]);
		});
		set.text_at(&Address::parse("brand#name-and-tagline").unwrap())
			.unwrap()
			.xpect_starts_with("The name is Acme Stalls.")
			.xpect_contains("```csv price-list\n");
	}

	/// A Word file under the documents directory reads like a markdown one:
	/// its title, tagline, summary and sections, its frontmatter from its core
	/// properties.
	#[beet::test]
	fn reads_a_word_document() {
		let mut set = set(vec![(
			"brand.docx",
			OoxmlFile::word(
				"<w:p><w:pPr><w:pStyle w:val=\"Title\"/></w:pPr><w:r><w:t>Acme Stalls</w:t></w:r></w:p>\
				 <w:p><w:r><w:rPr><w:i/></w:rPr><w:t>Stalls that come fitted</w:t></w:r></w:p>\
				 <w:p><w:r><w:t>Acme rents fitted market stalls.</w:t></w:r></w:p>\
				 <w:p><w:pPr><w:pStyle w:val=\"Heading2\"/></w:pPr><w:r><w:t>Voice</w:t></w:r></w:p>\
				 <w:p><w:r><w:t>Plain. TODO(ask): the voice.</w:t></w:r></w:p>",
			)
			.unwrap(),
		)]);
		let root = set.get("brand").unwrap().root;
		set.query(|query| {
			query.title(root).unwrap().xpect_eq("Acme Stalls");
			query
				.tagline(root)
				.unwrap()
				.xpect_eq("Stalls that come fitted");
			query
				.words(query.summary(root).unwrap())
				.xpect_eq("Acme rents fitted market stalls.");
			query.sections(root)[0].slug.as_str().xpect_eq("voice");
			query.asks(root)[0]
				.section
				.clone()
				.xpect_eq(Some("voice".into()));
		});
	}

	#[beet::test]
	fn resolves_promoted_documents() {
		let set = set(vec![
			(
				"market/index.md",
				MediaBytes::new_markdown("# Market\n\n## Research\n"),
			),
			(
				"market/customers.md",
				MediaBytes::new_markdown("# Customers\n"),
			),
			("brand.md", MediaBytes::new_markdown("# Brand\n")),
		]);
		set.files()
			.iter()
			.map(|document| document.name.as_str())
			.collect::<Vec<_>>()
			.xpect_eq(vec!["brand", "market/customers", "market"]);
		set.get("market")
			.unwrap()
			.file
			.as_str()
			.xpect_eq("market/index.md");
		DocumentSet::top_level(set.get("market/customers").unwrap())
			.xpect_eq("market");
		set.get("missing").xpect_none();
	}
}
