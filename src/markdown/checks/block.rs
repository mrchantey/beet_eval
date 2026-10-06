use crate::prelude::*;
use beet::prelude::*;

/// `check/block`: passes when the named `csv` block is defined exactly once
/// under the documents directory, in the section's document and, when it sits
/// in that document's own file, under that section, with exactly these
/// columns, every cell of a typed column of its type.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct BlockCheckParams {
	/// The block's name, its fence's info string after `csv`.
	pub name: SmolStr,
	/// Where it lives, ie `product#pricing`.
	pub section: Address,
	/// Its columns in order, each spelled as [`Column`] displays, ie
	/// `price_ex_gst:num`.
	pub columns: Vec<SmolStr>,
}
