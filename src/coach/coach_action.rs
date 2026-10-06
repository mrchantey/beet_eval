use crate::prelude::*;
use beet::prelude::*;

/// One of the coach's moves: a row of a document package's `actions` table,
/// keyed on its id, the question asked, how hard to push, and a bad and a good
/// exchange to calibrate by.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct CoachAction {
	/// A slug unique in the table, ie `status-quo`.
	pub id: SmolStr,
	/// What the move establishes, one sentence.
	pub statement: String,
	/// The question the coach asks.
	pub ask: String,
	/// When an answer is enough to stop pushing.
	pub push_until: String,
	/// Answers that mean push again.
	pub red_flags: Vec<String>,
	/// An exchange that fails the move.
	pub bad: String,
	/// An exchange that makes it.
	pub good: String,
	/// Where the move comes from.
	pub sources: Vec<SourceTag>,
	/// The sections of the documents its answers feed.
	pub serves: Vec<Address>,
}

/// A row of a document package's `actions` table, keyed on its id.
impl TableStoreRow for CoachAction {
	fn table_name() -> SmolStr { "actions".into() }
	fn key(&self) -> TableKey { self.id.as_str().into() }
}
