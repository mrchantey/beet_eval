use beet::prelude::*;

/// What every document's frontmatter carries, and nothing else.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct DocumentMeta {
	/// The day the document was first written.
	pub created: Date,
	/// The day of its last substantive edit, which a reader checks before
	/// trusting a figure and a grade is stale against.
	pub updated: Date,
	/// Who wrote it.
	pub authors: Vec<String>,
}

impl DocumentMeta {
	/// The keys, in the order a document writes them.
	pub const KEYS: [&str; 3] = ["created", "updated", "authors"];
}
