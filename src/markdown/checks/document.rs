use beet::prelude::*;

/// `check/document`: passes when the document exists, as `<document>.md` or
/// `<document>/index.md` under the documents directory, so promotion never
/// breaks it.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct DocumentCheckParams {
	/// The document's name, ie `brand`.
	pub document: SmolStr,
}
