use crate::prelude::*;
use beet::prelude::*;

/// One run over the subject: every check decided, the grades merged, and every
/// rubric read off the outcome. Written by `eval/results` to
/// `results/summary.json`, whose `--format=md` renders the same value for a
/// reader; never edited by hand.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct Results {
	/// The day of the run.
	pub ran_at: Date,
	/// Every checked eval's outcome, by id.
	pub checks: Vec<CheckOutcome>,
	/// The grades merged into the run, absent when no grader has written one.
	pub grades: Option<GradeSet>,
	/// Every rubric of every package, read off the checks and the grades.
	pub rubrics: Vec<RubricResult>,
}

impl Results {
	/// Where a workspace keeps the last run, below its results directory.
	pub const SUMMARY: &str = "summary.json";
}

/// The grades a run merged: the rows as they stood, and the rows it could not
/// use because the eval they grade has since been deleted, made checked, or
/// lost the level they give.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct GradeSet {
	/// The latest day any merged row was given.
	pub date: Date,
	/// Every grader of record among the rows, sorted.
	pub by: Vec<SmolStr>,
	/// The merged rows, by eval.
	pub rows: Vec<Grade>,
	/// Why each refused row was refused.
	pub problems: Vec<String>,
}

/// One rubric as a run reads it.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct RubricResult {
	/// The rubric, `<package>/<rubric>`.
	pub id: RubricRef,
	/// Every citation in heading order, with what the run found.
	pub citations: Vec<CitationResult>,
}

impl RubricResult {
	/// The rubric's line in a triage: how many citations are met, failing and
	/// awaiting.
	pub fn row(&self) -> RubricRow {
		let count = |status: fn(&CitationStatus) -> bool| {
			self.citations
				.iter()
				.filter(|citation| status(&citation.status))
				.count() as u32
		};
		RubricRow {
			rubric: self.id.clone(),
			met: count(|status| matches!(status, CitationStatus::Met)),
			failing: count(|status| {
				matches!(status, CitationStatus::Failing { .. })
			}),
			awaiting: count(|status| {
				matches!(status, CitationStatus::Awaiting)
			}),
		}
	}
}

/// One citation as a run reads it.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct CitationResult {
	/// The cited eval.
	pub eval: EvalId,
	/// The level the rubric demands.
	pub level: EvalLevel,
	/// What the run found.
	pub status: CitationStatus,
}

/// Whether a citation is met. A checked eval's citation is met when its check
/// passes, a judged eval's when its grade reaches the cited level.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub enum CitationStatus {
	/// The eval reached the cited level.
	Met,
	/// The eval fell short of it.
	Failing {
		/// The level reached: 0 for a failed check, else the grade's.
		got: EvalLevel,
	},
	/// A judged eval no grader has graded yet.
	Awaiting,
}
