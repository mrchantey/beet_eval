use beet::prelude::*;

/// `check/tagline`: passes when one emphasised line, `*like this*`, stands
/// alone beneath the title.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct TaglineCheckParams {
	/// The document's name.
	pub document: SmolStr,
}
