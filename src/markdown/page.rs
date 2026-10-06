use crate::prelude::*;
use beet::prelude::*;

/// One markdown page under the workspace's documents directory as the reader
/// parses it: its frontmatter, head, sections, data blocks and asks, with the
/// pages it was promoted into beneath it. The [`markdown`](crate::markdown)
/// module docs are its law.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct MarkdownPage {
	/// The page's address without a section, ie `market`, or
	/// `market/customers` for a promoted child.
	pub name: SmolStr,
	/// The file it was read from, relative to the documents directory, ie
	/// `market.md` or `market/index.md`.
	pub file: RelPath,
	/// The frontmatter, absent when the file has none.
	pub meta: Option<PageMeta>,
	/// The level one heading.
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
	/// The pages promoted out of this one's sections, read through it.
	pub children: Vec<MarkdownPage>,
}

impl MarkdownPage {
	/// The [`PageMeta`] keys every page's frontmatter carries, in the order a
	/// page writes them.
	pub const META_KEYS: [&str; 3] = ["created", "updated", "authors"];

	/// The section whose slug is `slug`.
	pub fn section(&self, slug: &str) -> Option<&Section> {
		self.sections.iter().find(|section| section.slug == slug)
	}
}
