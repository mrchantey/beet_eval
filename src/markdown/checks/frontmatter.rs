use beet::prelude::*;

/// `check/frontmatter`: passes when every document resolves and its
/// frontmatter carries every key with a value.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct FrontmatterCheckParams {
	/// The keys each must carry, ie
	/// [`MarkdownPage::META_KEYS`](crate::prelude::MarkdownPage::META_KEYS).
	pub keys: Vec<SmolStr>,
	/// The documents' names.
	pub documents: Vec<SmolStr>,
}
