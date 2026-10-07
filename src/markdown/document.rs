use crate::prelude::*;
use beet::exports::bevy::reflect::TypeRegistry;
use beet::prelude::*;
use pulldown_cmark::CodeBlockKind;
use pulldown_cmark::Event;
use pulldown_cmark::HeadingLevel;
use pulldown_cmark::Parser;
use pulldown_cmark::Tag;
use pulldown_cmark::TagEnd;
use std::sync::OnceLock;

/// One markdown document under the workspace's documents directory as the reader
/// parses it: its frontmatter, head, sections, data blocks and asks, with the
/// documents it was promoted into beneath it. The [`markdown`](crate::markdown)
/// module docs are its law.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct MarkdownDocument {
	/// The document's address without a section, ie `market`, or
	/// `market/customers` for a promoted child.
	pub name: SmolStr,
	/// The file it was read from, relative to the documents directory, ie
	/// `market.md` or `market/index.md`.
	pub file: RelPath,
	/// The frontmatter, absent when the file has none.
	pub meta: Option<PageMeta>,
	/// The level one heading as written, when it opens the body.
	pub title: Option<String>,
	/// The emphasised line beneath the title, emphasis removed.
	pub tagline: Option<String>,
	/// The paragraph between the head and the first section.
	pub summary: Option<String>,
	/// The `##` sections, in order.
	pub sections: Vec<Section>,
	/// Every named block in the file, in order.
	pub blocks: Vec<DataBlock>,
	/// Every ask in the file, in order.
	pub asks: Vec<Ask>,
	/// The documents promoted out of this one's sections, read through it.
	pub children: Vec<MarkdownDocument>,
	/// What could not be read, ie frontmatter that does not parse.
	pub problems: Vec<String>,
}

impl MarkdownDocument {
	/// The [`PageMeta`] keys every document's frontmatter carries, in the order
	/// a document writes them.
	pub const META_KEYS: [&str; 3] = ["created", "updated", "authors"];

	/// Reads one file, `name` its address and `file` its path under the
	/// documents directory. The structure comes from the markdown parse, so a
	/// heading inside a fence is no heading; a line's own rules (a tagline's
	/// emphasis, a block's info string, an ask) are read off its source.
	pub fn parse(
		name: impl Into<SmolStr>,
		file: RelPath,
		source: &str,
	) -> Result<Self> {
		let (meta, body, body_line) = Self::split_frontmatter(source);
		let mut problems = Vec::new();
		let meta = meta.unwrap_or_else(|err| {
			problems.push(format!("the frontmatter does not parse: {err}"));
			None
		});
		let mut document = Self {
			name: name.into(),
			file,
			meta,
			title: None,
			tagline: None,
			summary: None,
			sections: Vec::new(),
			blocks: Vec::new(),
			asks: Vec::new(),
			children: Vec::new(),
			problems,
		};
		let lines = LineIndex::new(body, body_line);
		let mut blocks = TopBlock::collect(body);
		// the head: a title as the first block, a tagline straight beneath it
		let mut head_end = 0;
		if let Some(TopBlock::Heading { level: 1, range }) = blocks.first() {
			document.title = Some(Self::heading_text(&body[range.clone()]));
			head_end = 1;
			if let Some(TopBlock::Paragraph { range }) = blocks.get(1) {
				if let Some(tagline) = Self::tagline_of(&body[range.clone()]) {
					document.tagline = Some(tagline);
					head_end = 2;
				}
			}
		}
		// the summary: the first paragraph before the first section
		document.summary = blocks
			.iter()
			.skip(head_end)
			.take_while(|block| !block.is_section())
			.find_map(|block| match block {
				TopBlock::Paragraph { range } => {
					Some(body[range.clone()].trim().to_string())
				}
				_ => None,
			});
		// the sections, each running to the next
		let starts = blocks
			.iter()
			.filter_map(|block| match block {
				TopBlock::Heading { level: 2, range } => Some(range.clone()),
				_ => None,
			})
			.collect::<Vec<_>>();
		for (index, range) in starts.iter().enumerate() {
			let heading = Self::heading_text(&body[range.clone()]);
			let end =
				starts.get(index + 1).map_or(body.len(), |next| next.start);
			document.sections.push(Section {
				slug: Section::slug(&heading),
				heading,
				line: lines.line(range.start),
				body: body[range.end.min(end)..end].trim().to_string(),
			});
		}
		// the named blocks, at any depth
		for block in blocks.drain(..) {
			if let TopBlock::Code { info, range, text } = block {
				if let Some((format, block_name)) = DataBlock::parse_info(&info)
				{
					let line = lines.line(range.start);
					document.blocks.push(DataBlock::new(
						block_name,
						format,
						line,
						document.section_at(line),
						text,
					));
				}
			}
		}
		// the asks, line by line
		for (offset, text) in body.split('\n').enumerate() {
			let line = body_line + offset as u32 + 1;
			for (kind, question) in Ask::find_all(text) {
				document.asks.push(Ask {
					kind,
					text: question,
					line,
					section: document.section_at(line),
				});
			}
		}
		document.xok()
	}

	/// The section whose slug is `slug`.
	pub fn section(&self, slug: &str) -> Option<&Section> {
		self.sections.iter().find(|section| section.slug == slug)
	}

	/// Everything written in the document, its head and every section, one
	/// block to a paragraph: what a quote anchored at the whole document is
	/// looked for in.
	pub fn text(&self) -> String {
		self.title
			.iter()
			.chain(&self.tagline)
			.chain(&self.summary)
			.cloned()
			.chain(self.sections.iter().map(|section| {
				format!("{}\n\n{}", section.heading, section.body)
			}))
			.collect::<Vec<_>>()
			.join("\n\n")
	}

	/// This document and every document promoted out of it, depth first.
	pub fn family(&self) -> Vec<&MarkdownDocument> {
		std::iter::once(self)
			.chain(self.children.iter().flat_map(MarkdownDocument::family))
			.collect()
	}

	/// The slug of the section a line falls in, absent above the first.
	fn section_at(&self, line: u32) -> Option<SmolStr> {
		self.sections
			.iter()
			.take_while(|section| section.line <= line)
			.last()
			.map(|section| section.slug.clone())
	}

	/// A heading's text as written, its `#` marker and spacing removed.
	fn heading_text(source: &str) -> String {
		source
			.lines()
			.next()
			.unwrap_or_default()
			.trim_start()
			.trim_start_matches('#')
			.trim()
			.to_string()
	}

	/// A one-line paragraph wholly emphasised, `*like this*`, its markers
	/// removed.
	fn tagline_of(source: &str) -> Option<String> {
		let line = source.trim();
		let marked = |char: char| char == '*' || char == '_';
		(!line.contains('\n')
			&& line.len() > 2
			&& line.starts_with(marked)
			&& line.ends_with(marked))
		.then(|| line.trim_matches(marked).trim().to_string())
	}

	/// The frontmatter read into [`PageMeta`], the body after it, and the
	/// number of file lines before the body. An unclosed fence is no
	/// frontmatter.
	fn split_frontmatter(
		source: &str,
	) -> (Result<Option<PageMeta>>, &str, u32) {
		let first = source.lines().next().unwrap_or_default().trim();
		let kind = match first {
			"---" => FrontmatterKind::Yaml,
			"+++" => FrontmatterKind::Toml,
			_ => return (Ok(None), source, 0),
		};
		let content_start = source.find('\n').map_or(source.len(), |at| at + 1);
		let mut offset = content_start;
		for (index, line) in source[content_start..].split('\n').enumerate() {
			if line.trim() == first {
				let content = &source[content_start..offset];
				let body_start = (offset + line.len() + 1).min(source.len());
				let meta =
					Frontmatter::parse(content, kind).and_then(|frontmatter| {
						frontmatter
							.declarations(&type_ext::short_name::<PageMeta>())
							.get::<PageMeta>(Self::meta_registry())
					});
				return (meta, &source[body_start..], index as u32 + 2);
			}
			offset += line.len() + 1;
		}
		(Ok(None), source, 0)
	}

	/// The registry frontmatter is read against, holding [`PageMeta`].
	fn meta_registry() -> &'static TypeRegistry {
		static REGISTRY: OnceLock<TypeRegistry> = OnceLock::new();
		REGISTRY.get_or_init(|| {
			let mut registry = TypeRegistry::default();
			registry.register::<PageMeta>();
			registry
		})
	}
}

/// A block of the parse as the reader needs it: its kind and its byte range
/// in the body.
enum TopBlock {
	Heading {
		level: u8,
		range: std::ops::Range<usize>,
	},
	Paragraph {
		range: std::ops::Range<usize>,
	},
	/// A fenced code block at any depth, with its info string and text.
	Code {
		info: String,
		range: std::ops::Range<usize>,
		text: String,
	},
	/// Any other top level block.
	Other,
}

impl TopBlock {
	/// The top level headings, paragraphs and other blocks in order, with
	/// every fenced code block wherever it sits.
	fn collect(body: &str) -> Vec<Self> {
		let mut blocks = Vec::new();
		let mut depth = 0usize;
		let mut code: Option<(String, std::ops::Range<usize>, String)> = None;
		for (event, range) in Parser::new(body).into_offset_iter() {
			match event {
				Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => {
					code = Some((info.to_string(), range, String::new()));
					depth += 1;
				}
				Event::Text(text) if code.is_some() => {
					code.as_mut().unwrap().2.push_str(&text);
				}
				Event::End(TagEnd::CodeBlock) => {
					depth -= 1;
					if let Some((info, range, text)) = code.take() {
						// the body as written, without the closing line break
						let text = text
							.strip_suffix('\n')
							.unwrap_or(&text)
							.to_string();
						blocks.push(Self::Code { info, range, text });
					}
				}
				Event::Start(tag) => {
					if depth == 0 {
						blocks.push(match tag {
							Tag::Heading { level, .. } => Self::Heading {
								level: Self::level(level),
								range,
							},
							Tag::Paragraph => Self::Paragraph { range },
							_ => Self::Other,
						});
					}
					depth += 1;
				}
				Event::End(_) => depth -= 1,
				_ => {}
			}
		}
		blocks
	}

	fn level(level: HeadingLevel) -> u8 {
		match level {
			HeadingLevel::H1 => 1,
			HeadingLevel::H2 => 2,
			HeadingLevel::H3 => 3,
			HeadingLevel::H4 => 4,
			HeadingLevel::H5 => 5,
			HeadingLevel::H6 => 6,
		}
	}

	fn is_section(&self) -> bool {
		matches!(self, Self::Heading { level: 2, .. })
	}
}

/// Byte offsets of a body to the file lines they fall on, from 1.
struct LineIndex {
	/// The byte offset each body line starts at.
	starts: Vec<usize>,
	/// The file lines before the body.
	before: u32,
}

impl LineIndex {
	fn new(body: &str, before: u32) -> Self {
		let starts = std::iter::once(0)
			.chain(body.match_indices('\n').map(|(at, _)| at + 1))
			.collect();
		Self { starts, before }
	}

	fn line(&self, offset: usize) -> u32 {
		let index = self.starts.partition_point(|start| *start <= offset);
		self.before + index as u32
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

	#[beet::test]
	fn reads_a_document() {
		let document =
			MarkdownDocument::parse("brand", RelPath::new("brand.md"), BRAND)
				.unwrap();
		let meta = document.meta.clone().unwrap();
		meta.created.unwrap().to_string().xpect_eq("2026-10-02");
		meta.authors.xpect_eq(vec![SmolStr::new("Ada Lovelace")]);
		document.title.clone().unwrap().xpect_eq("Acme Stalls");
		document
			.tagline
			.clone()
			.unwrap()
			.xpect_eq("Stalls that come fitted");
		document
			.summary
			.clone()
			.unwrap()
			.xpect_eq("Acme rents fitted market stalls by the month.");
		document
			.sections
			.iter()
			.map(|section| (section.slug.as_str(), section.line))
			.collect::<Vec<_>>()
			.xpect_eq(vec![("name-and-tagline", 13), ("voice", 22)]);
		document
			.section("name-and-tagline")
			.unwrap()
			.body
			.clone()
			.xpect_starts_with("The name is Acme Stalls.")
			.xpect_ends_with("```");
		let block = &document.blocks[0];
		(block.name.as_str(), block.line, block.section.clone()).xpect_eq((
			"price-list",
			17,
			Some(SmolStr::new("name-and-tagline")),
		));
		block
			.rows
			.clone()
			.xpect_eq(vec![vec!["Stall, fitted".to_string(), "120".into()]]);
		document.blocks.len().xpect_eq(1);
		document
			.asks
			.iter()
			.map(|ask| (ask.kind, ask.text.as_str(), ask.line))
			.collect::<Vec<_>>()
			.xpect_eq(vec![
				(AskKind::Fact, "the tagline's reasoning.", 15),
				(AskKind::Decision, "playful or plain?", 24),
				(AskKind::Fact, "who reads it", 24),
			]);
	}
}
