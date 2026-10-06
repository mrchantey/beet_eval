use crate::prelude::*;
use beet::prelude::*;

/// `check/agrees`: passes when the document's title or tagline appears
/// verbatim, emphasis aside, in the section of the document that owns it, or
/// when both sides carry the same open ask, so an undecided value is reported
/// by `check/asks` alone. The index is the source of truth for both.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct AgreesCheckParams {
	/// The document whose title or tagline is the source of truth, ie `index`.
	pub document: SmolStr,
	/// Which of the two must agree.
	pub part: TitlePart,
	/// The section that owns it in detail, ie `brand#name-and-tagline`.
	pub section: Address,
}

/// The two parts of a document's head that another section may own.
#[derive(
	Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize,
)]
pub enum TitlePart {
	/// The level one heading.
	Title,
	/// The emphasised line beneath it.
	Tagline,
}
