use beet::prelude::*;

/// `check/sections`: passes when the document's `##` headings are exactly
/// these, in this order, and names the missing, the extra or the misordered.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct SectionsCheckParams {
	/// The document's name.
	pub document: SmolStr,
	/// The headings, as written.
	pub headings: Vec<String>,
}
