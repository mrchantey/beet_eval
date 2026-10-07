use crate::prelude::*;
use beet::prelude::*;

/// One grader's level for one judged eval, with the verbatim evidence it rests
/// on: a row of the workspace's `results/grades` table, keyed on the eval, so a
/// regrade replaces the row.
///
/// Written only through `eval/grade`, which refuses an unknown or checked eval,
/// a level the eval's [`Levels`] do not admit, and evidence that is not a
/// verbatim substring of the anchored section.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct Grade {
	/// The judged eval.
	pub eval: EvalId,
	/// The level by the eval's own lines, the lower of two when between them.
	pub level: EvalLevel,
	/// Where the evidence sits, or where it was looked for at level 0.
	pub anchor: Address,
	/// A verbatim quote of at most twenty-five words from the anchor; absent
	/// only when nothing is written there.
	pub evidence: Option<SmolStr>,
	/// The day the grade was given, which a document's `updated` is compared
	/// against to find a stale grade.
	pub date: Date,
	/// The grader of record, a model or a person.
	pub by: SmolStr,
}

/// A row of the workspace's `grades` table, keyed on the eval it grades.
impl TableStoreRow for Grade {
	fn table_name() -> SmolStr { "grades".into() }
	fn key(&self) -> TableKey { self.eval.as_str().into() }
}

impl Grade {
	/// The most words a quote may carry.
	pub const MAX_EVIDENCE_WORDS: usize = 25;
}
