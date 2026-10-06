use crate::prelude::*;
use beet::prelude::*;

/// Where the subject needs work next: one unit of work, why, and the tables the
/// choice was made from. `eval/next` computes it from the same run as
/// [`Results`], in this order:
///
/// 1. [`Scaffold`](NextVerb::Scaffold) when no document of the outline exists.
/// 2. [`Write`](NextVerb::Write) the first missing document, then the document
///    failing the most shape checks: shape before substance.
/// 3. [`Grade`](NextVerb::Grade) every document written with no asks open but
///    ungraded, or graded before its last `updated`.
/// 4. [`Build`](NextVerb::Build) the first reader rubric fully met, with a
///    [`RenderSpec`] and nothing built for it.
/// 5. [`Write`](NextVerb::Write) the document with the most open asks and the
///    widest gap to the owner rubric, weighing an ask as two levels.
/// 6. [`Done`](NextVerb::Done) when nothing is left.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct NextStep {
	/// The unit of work.
	pub verb: NextVerb,
	/// What it is done to: a document, the documents to grade, a
	/// `<package>/<rubric>` to build, a claim to coach, `docs/` to scaffold;
	/// empty when done.
	pub targets: Vec<SmolStr>,
	/// Why this unit and not another, one paragraph.
	pub why: String,
	/// Every document of the outline, in its order.
	pub documents: Vec<DocumentRow>,
	/// Every rubric of every package.
	pub rubrics: Vec<RubricRow>,
}

/// The kinds of unit of work, each a role's turn.
#[derive(
	Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize,
)]
pub enum NextVerb {
	/// `eval/new` lays out the documents from the outline, every body an ask.
	Scaffold,
	/// The clerk writes a document from the sources and the owner's answers.
	Write,
	/// The grader grades documents against the worksheet.
	Grade,
	/// The builder fills a reader's form.
	Build,
	/// The coach runs a sitting on a claim.
	Coach,
	/// Nothing is left.
	Done,
}

/// One document as the triage weighs it.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct DocumentRow {
	/// The document's name in the outline.
	pub document: SmolStr,
	/// Whether it exists, as a file or a directory with an index.
	pub present: bool,
	/// The asks open in it and its children.
	pub asks: u32,
	/// The failing checks that concern its shape.
	pub shape_failures: Vec<EvalId>,
	/// The levels still missing to the owner rubric, over the evals anchored
	/// here.
	pub owner_gap: u32,
	/// The same over every reader rubric.
	pub reader_gap: u32,
	/// How many of the judged evals anchored here are graded.
	pub graded: u32,
	/// How many judged evals are anchored here.
	pub judged: u32,
	/// Its frontmatter's `updated`.
	pub updated: Option<Date>,
	/// Whether it changed after its grades were given.
	pub stale: bool,
}

/// One rubric as the triage counts it.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct RubricRow {
	/// The rubric, `<package>/<rubric>`.
	pub rubric: RubricRef,
	/// Citations met.
	pub met: u32,
	/// Citations failing.
	pub failing: u32,
	/// Citations awaiting a grader.
	pub awaiting: u32,
}
