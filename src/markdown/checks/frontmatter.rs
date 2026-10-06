use beet::prelude::*;

/// `check/frontmatter`: passes when every document resolves and its
/// frontmatter carries every key with a value.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct FrontmatterCheckParams {
	/// The keys each must carry, ie
	/// [`MarkdownDocument::META_KEYS`](crate::prelude::MarkdownDocument::META_KEYS).
	pub keys: Vec<SmolStr>,
	/// The documents' names.
	pub documents: Vec<SmolStr>,
}
