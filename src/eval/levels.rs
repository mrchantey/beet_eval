use crate::prelude::*;
use beet::prelude::*;

/// How an [`Eval`] is decided.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub enum Levels {
	/// The scale's own words, [`EvalLevel::meaning`], apply to the statement as
	/// written.
	Generic,
	/// Met or not: [`EvalLevel::MISSING`] or [`EvalLevel::SOUND`] only, decided
	/// by the check route when there is one and by a grader otherwise.
	Binary {
		/// The check route that decides it, absent for a judged binary eval.
		check: Option<CheckRef>,
	},
	/// All four lines pinned because the scale's words are not precise enough,
	/// each one sentence, each strictly harder than the last.
	Custom {
		/// What level 0 means for this eval.
		l0: String,
		/// What level 1 means for this eval.
		l1: String,
		/// What level 2 means for this eval.
		l2: String,
		/// What level 3 means for this eval.
		l3: String,
	},
}

impl Levels {
	/// The check route deciding the eval, if any.
	pub fn check(&self) -> Option<&CheckRef> {
		match self {
			Self::Binary { check } => check.as_ref(),
			_ => None,
		}
	}

	/// Whether `level` is one this eval can be given: 0 or 2 for a binary
	/// eval, any of the four otherwise.
	pub fn admits(&self, level: EvalLevel) -> bool {
		match self {
			Self::Binary { .. } => {
				level == EvalLevel::MISSING || level == EvalLevel::SOUND
			}
			_ => true,
		}
	}

	/// What `level` means for this eval: its own pinned line when custom, the
	/// scale's words otherwise.
	pub fn line(&self, level: EvalLevel) -> &str {
		match (self, level.get()) {
			(Self::Custom { l0, .. }, 0) => l0,
			(Self::Custom { l1, .. }, 1) => l1,
			(Self::Custom { l2, .. }, 2) => l2,
			(Self::Custom { l3, .. }, _) => l3,
			_ => level.meaning(),
		}
	}
}

/// One step of the four-level scale every judged eval is graded on, 0 to 3; a
/// rubric names the level each citation must reach.
#[derive(
	Debug,
	Clone,
	Copy,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Hash,
	Reflect,
	Serialize,
	Deserialize,
)]
#[serde(try_from = "u8", into = "u8")]
pub struct EvalLevel(u8);

impl EvalLevel {
	/// Nothing is written, or only a placeholder, a category or a restated
	/// prompt.
	pub const MISSING: Self = Self(0);
	/// An answer exists but is generic, unsupported or incomplete; it could
	/// belong to any subject.
	pub const STATED: Self = Self(1);
	/// Specific to this subject, complete, with a reason or a source; what an
	/// assessor or a coach accepts. A passing check awards it.
	pub const SOUND: Self = Self(2);
	/// Evidenced, quantified and tested against alternatives; what an investor,
	/// a partner or the author a year later would hold it to.
	pub const STRONG: Self = Self(3);

	/// Validates `level` as 0 to 3.
	pub fn new(level: u8) -> Result<Self> {
		match level {
			0..=3 => Self(level).xok(),
			_ => bevybail!("level {level} is not 0 to 3"),
		}
	}

	/// The level as a number.
	pub fn get(&self) -> u8 { self.0 }

	/// The scale's words for this level.
	pub fn meaning(&self) -> &'static str {
		match self.0 {
			0 => {
				"Missing: nothing is written, or only a placeholder, a category \
				 or a restated prompt."
			}
			1 => {
				"Stated: an answer exists but is generic, unsupported or \
				 incomplete; it could belong to any subject."
			}
			2 => {
				"Sound: specific to this subject, complete, with a reason or a \
				 source; what an assessor or a coach accepts."
			}
			_ => {
				"Strong: evidenced, quantified and tested against alternatives; \
				 what an investor, a partner or the author a year later would \
				 hold it to."
			}
		}
	}
}

impl TryFrom<u8> for EvalLevel {
	type Error = BevyError;
	fn try_from(level: u8) -> Result<Self> { Self::new(level) }
}

impl From<EvalLevel> for u8 {
	fn from(level: EvalLevel) -> u8 { level.0 }
}

impl core::fmt::Display for EvalLevel {
	fn fmt(
		&self,
		formatter: &mut core::fmt::Formatter<'_>,
	) -> core::fmt::Result {
		self.0.fmt(formatter)
	}
}
