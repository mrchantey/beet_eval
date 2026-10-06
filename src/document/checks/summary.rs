use beet::prelude::*;

/// `check/summary`: passes when every document has a paragraph between its
/// title, or tagline, and its first `##`.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct SummaryCheckParams {
	/// The documents' names.
	pub documents: Vec<SmolStr>,
}
