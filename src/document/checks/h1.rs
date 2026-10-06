use beet::prelude::*;

/// `check/h1`: passes when the first non-blank line after the frontmatter is a
/// level one heading.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct H1CheckParams {
	/// The document's name.
	pub document: SmolStr,
}
