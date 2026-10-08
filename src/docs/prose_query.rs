use crate::prelude::*;
use beet::prelude::*;

/// The traversals that read a document's tree, whatever file it was parsed
/// from: a markdown file and a Word file read alike, since both parse into
/// the same HTML terms. Every method takes the document's root.
///
/// - **Blocks**: the document's top level, its paragraphs, headings, lists,
///   tables and code blocks, read through every node that means nothing to a
///   reader, ie a Word file's part and body, and past a page's `<header>`,
///   `<footer>` and notes.
/// - **Title**: the first block, when it is an `<h1>`.
/// - **Tagline**: the block beneath the title, when it is a paragraph whose
///   words are one emphasis, `*like this*`.
/// - **Summary**: the first paragraph after the head and before the first
///   section.
/// - **Sections**: each `<h2>` and the blocks after it, up to the next, its
///   slug [`Address::slug`] of its heading's words.
/// - **Data blocks**: every `pre > code` whose `data-info` names a block,
///   `csv price-list`, at any depth.
/// - **Asks**: every `TODO(ask ..)` in a paragraph's words.
/// - **Frontmatter**: the root's [`PageMeta`], from a markdown file's
///   frontmatter or a Word file's core properties.
#[derive(SystemParam)]
pub struct ProseQuery<'w, 's> {
	nodes: Query<
		'w,
		's,
		(
			Option<&'static Element>,
			Option<&'static Value>,
			Option<&'static Children>,
		),
	>,
	spans: Query<'w, 's, &'static FileSpan>,
	parents: Query<'w, 's, &'static ChildOf>,
	metas: Query<'w, 's, &'static PageMeta>,
	attributes: AttributeQuery<'w, 's>,
	text: ReaderText<'w, 's>,
}

/// One `##` section of a document: its heading, the slug that addresses it,
/// and the blocks it holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSection {
	/// The heading's words, ie `Owner's finances`.
	pub heading: SmolStr,
	/// The heading's slug, the section half of an [`Address`], ie
	/// `owners-finances`.
	pub slug: SmolStr,
	/// The heading's line in its file, from 1, or its block's place in a file
	/// of no lines.
	pub line: u32,
	/// The blocks after the heading, up to the next.
	pub blocks: Vec<Entity>,
}

impl ProseQuery<'_, '_> {
	/// What a page wraps around its content, and an alternative no reader
	/// sees, which no document block is.
	const AROUND: &[&str] = &["header", "footer", "aside", "nav", "template"];

	/// The document's frontmatter.
	pub fn meta(&self, root: Entity) -> Option<&PageMeta> {
		self.metas.get(root).ok()
	}

	/// The document's top-level blocks, in order.
	pub fn blocks(&self, root: Entity) -> Vec<Entity> {
		let mut out = Vec::new();
		self.push_blocks(root, &mut out);
		out
	}

	/// The title: the first block's words, when it is an `<h1>`.
	pub fn title(&self, root: Entity) -> Option<SmolStr> {
		let first = *self.blocks(root).first()?;
		(self.tag(first) == Some("h1")).then(|| self.words(first).into())
	}

	/// The tagline: the words of the paragraph beneath the title, when they
	/// are one emphasis.
	pub fn tagline(&self, root: Entity) -> Option<SmolStr> {
		self.title(root)?;
		let block = *self.blocks(root).get(1)?;
		self.emphasis_only(block).then(|| self.words(block).into())
	}

	/// The summary: the first paragraph after the head and before the first
	/// section.
	pub fn summary(&self, root: Entity) -> Option<Entity> {
		let head = match (self.title(root), self.tagline(root)) {
			(None, _) => 0,
			(Some(_), None) => 1,
			(Some(_), Some(_)) => 2,
		};
		self.blocks(root)
			.into_iter()
			.skip(head)
			.take_while(|block| self.tag(*block) != Some("h2"))
			.find(|block| {
				self.tag(*block) == Some("p") && !self.words(*block).is_empty()
			})
	}

	/// The `##` sections, in order.
	pub fn sections(&self, root: Entity) -> Vec<DocumentSection> {
		let mut sections = Vec::<DocumentSection>::new();
		for block in self.blocks(root) {
			match (self.tag(block), sections.last_mut()) {
				(Some("h2"), _) => {
					let heading = self.words(block);
					sections.push(DocumentSection {
						slug: Address::slug(&heading),
						heading: heading.into(),
						line: self.line(root, block),
						blocks: Vec::new(),
					});
				}
				(_, Some(section)) => section.blocks.push(block),
				(_, None) => {}
			}
		}
		sections
	}

	/// The section whose slug is `slug`.
	pub fn section(&self, root: Entity, slug: &str) -> Option<DocumentSection> {
		self.sections(root)
			.into_iter()
			.find(|section| section.slug == slug)
	}

	/// The section `entity` sits in, absent above the first.
	pub fn section_of(&self, root: Entity, entity: Entity) -> Option<SmolStr> {
		let top = core::iter::once(entity)
			.chain(self.parents.iter_ancestors(entity))
			.find(|ancestor| self.blocks(root).contains(ancestor))?;
		self.sections(root)
			.into_iter()
			.find(|section| section.blocks.contains(&top))
			.map(|section| section.slug)
	}

	/// Every named data block, in document order at any depth.
	pub fn data_blocks(&self, root: Entity) -> Vec<DataBlock> {
		self.descendants(root)
			.into_iter()
			.filter(|entity| self.tag(*entity) == Some("code"))
			.filter_map(|code| {
				let pre = self
					.parents
					.get(code)
					.ok()
					.map(ChildOf::parent)
					.filter(|pre| self.tag(*pre) == Some("pre"))?;
				let info = self
					.attributes
					.find(code, "data-info")
					.map(|(_, value)| value.to_string())?;
				let (format, name) = DataBlock::parse_info(&info)?;
				let text = self.text.text(code);
				let text = text.strip_suffix('\n').unwrap_or(&text).to_string();
				DataBlock::new(
					name,
					format,
					self.line(root, pre),
					self.section_of(root, pre),
					text,
				)
				.xsome()
			})
			.collect()
	}

	/// Every ask in the document's paragraphs, in order, each with its line
	/// and its section.
	pub fn asks(&self, root: Entity) -> Vec<Ask> {
		let mut asks = Vec::new();
		for block in self.paragraphs(root) {
			let start = self.line(root, block);
			for (offset, line) in self.text.text(block).split('\n').enumerate()
			{
				for (kind, question) in Ask::find_all(line) {
					asks.push(Ask {
						kind,
						text: question,
						line: start + offset as u32,
						section: self.section_of(root, block),
					});
				}
			}
		}
		asks
	}

	/// The words a reader reads in `entity`, trimmed.
	pub fn words(&self, entity: Entity) -> String {
		self.text.text(entity).trim().to_string()
	}

	/// The line `entity` starts on in its file, from 1; a file of no lines,
	/// ie a Word file, numbers its top-level blocks instead.
	pub fn line(&self, root: Entity, entity: Entity) -> u32 {
		if let Ok(span) = self.spans.get(entity) {
			return span.start_line();
		}
		let blocks = self.blocks(root);
		core::iter::once(entity)
			.chain(self.parents.iter_ancestors(entity))
			.find_map(|ancestor| {
				blocks.iter().position(|block| *block == ancestor)
			})
			.map_or(0, |index| index as u32 + 1)
	}

	/// The paragraphs a reader reads words in, at any depth, each holding no
	/// paragraph of its own: a cell or a tight list item holding its words
	/// directly counts.
	fn paragraphs(&self, root: Entity) -> Vec<Entity> {
		self.descendants(root)
			.into_iter()
			.filter(|entity| {
				matches!(
					self.tag(*entity),
					Some(
						"p" | "h1"
							| "h2" | "h3" | "h4" | "h5"
							| "h6" | "li" | "td" | "th"
					)
				)
			})
			.filter(|entity| self.text.blocks(*entity).is_empty())
			.filter(|entity| {
				// a cell's paragraphs are paragraphs of their own
				!self.descendants(*entity).iter().any(|inner| {
					matches!(self.tag(*inner), Some("p" | "li" | "td" | "th"))
				})
			})
			.collect()
	}

	fn push_blocks(&self, entity: Entity, out: &mut Vec<Entity>) {
		for child in self.children(entity) {
			match self.tag(child) {
				Some(tag) if Self::AROUND.contains(&tag) => {}
				Some(_) => out.push(child),
				None => self.push_blocks(child, out),
			}
		}
	}

	/// Whether a block's words are one emphasis, everything else in it
	/// meaning nothing to a reader.
	fn emphasis_only(&self, block: Entity) -> bool {
		if self.tag(block) != Some("p") {
			return false;
		}
		let content = self.content(block);
		matches!(content.as_slice(), [only] if matches!(self.tag(*only), Some("em" | "i")))
	}

	/// The children of `entity` a reader reads, through every node that
	/// means nothing.
	fn content(&self, entity: Entity) -> Vec<Entity> {
		let mut out = Vec::new();
		for child in self.children(entity) {
			match self.nodes.get(child) {
				Ok((Some(_), ..)) => out.push(child),
				Ok((None, Some(Value::Str(text)), _))
					if !text.trim().is_empty() =>
				{
					out.push(child)
				}
				Ok((None, ..)) => out.extend(self.content(child)),
				Err(_) => {}
			}
		}
		out
	}

	fn tag(&self, entity: Entity) -> Option<&str> {
		self.nodes
			.get(entity)
			.ok()
			.and_then(|(element, ..)| element)
			.map(Element::tag)
	}

	fn children(&self, entity: Entity) -> Vec<Entity> {
		self.nodes
			.get(entity)
			.ok()
			.and_then(|(.., children)| children)
			.map(|children| children.to_vec())
			.unwrap_or_default()
	}

	fn descendants(&self, entity: Entity) -> Vec<Entity> {
		let mut out = Vec::new();
		for child in self.children(entity) {
			out.push(child);
			out.extend(self.descendants(child));
		}
		out
	}
}
