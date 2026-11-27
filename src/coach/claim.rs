use crate::prelude::*;
use beet::prelude::*;

/// One thing the subject's plan rests on that might be false: a row of the
/// workspace package's `claims` register, keyed on its id.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct Claim {
	/// A slug unique in the register, ie `buyers-pay-monthly`, one path
	/// segment, so the mount `coach/{claim}` names its sittings' thread.
	pub id: SmolStr,
	/// The claim, one sentence that could turn out false.
	pub claim: String,
	/// The section of the documents that rests on it.
	pub block: Address,
	/// How far it has been tested, the [`Evidence`] ladder.
	pub evidence: Evidence,
	/// What it costs the plan if the claim is false.
	pub stakes: Stakes,
	/// The cheapest test that would move its evidence, which the sitting that
	/// picked the claim assigns.
	pub test: String,
	/// Where it stands.
	pub status: ClaimStatus,
	/// Where the evidence came from.
	pub sources: Vec<SourceTag>,
}

impl Claim {
	/// The claim a sitting is held on: of those still in question, open or
	/// testing, the highest stakes, then the least evidence, then an open
	/// one before one already testing, then by id so the pick is stable.
	pub fn pick<'a>(
		claims: impl IntoIterator<Item = &'a Claim>,
	) -> Option<&'a Claim> {
		claims
			.into_iter()
			.filter(|claim| claim.status.in_question())
			.min_by(|left, right| {
				(left.stakes, left.evidence, left.status, &left.id).cmp(&(
					right.stakes,
					right.evidence,
					right.status,
					&right.id,
				))
			})
	}

	/// Whether a sitting is due on it: fatal, with evidence no more than
	/// heard.
	pub fn needs_coaching(&self) -> bool {
		self.stakes == Stakes::Fatal && self.evidence <= Evidence::HEARD
	}

	/// Whether it rests in the section of `document` with the slug
	/// `section`, or in the document itself when `section` is absent.
	pub fn rests_in(&self, document: &str, section: Option<&str>) -> bool {
		self.block.document_name() == document
			&& self.block.section() == section
	}

	/// Whether `id` is a claim id: a slug of lowercase letters, digits and
	/// dashes, one path segment.
	pub fn is_id(id: &str) -> bool {
		!id.is_empty()
			&& !id.starts_with('-')
			&& id.chars().all(|char| {
				char.is_ascii_lowercase()
					|| char.is_ascii_digit()
					|| char == '-'
			})
	}
}

/// A row of the workspace package's `claims` register, keyed on its id.
impl TableStoreRow for Claim {
	fn table_name() -> SmolStr { "claims".into() }
	fn key(&self) -> TableKey { self.id.as_str().into() }
}

/// How far a [`Claim`] has been tested, 0 to 3, each rung what someone gave
/// to stand behind it.
///
/// | Rung | Meaning |
/// |---|---|
/// | 0 | Asserted: the owner believes it, with nothing behind it. |
/// | 1 | Heard: someone it concerns said so, an opinion, a compliment or a promise about the future, which costs them nothing. |
/// | 2 | Observed: seen in what people did, their past behaviour, workarounds and spending. |
/// | 3 | Committed: someone gave something up to act on it, time, reputation or money. |
///
/// The ladder is The Mom Test's [ref:fitzpatrick-2013]: compliments and
/// hypotheticals are no data, facts about past behaviour are, and commitment
/// in its three currencies of time, reputation and money is the real signal.
/// Seibel's how to talk to users \[ref:seibel\] asks about the last time
/// rather than the next, the rung between heard and observed, and Blank's
/// customer validation [ref:blank-2005] and Maurya's validated learning
/// [ref:maurya-2012] hold a claim proven only by what customers commit to.
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
pub struct Evidence(u8);

impl Evidence {
	/// The owner believes it, with nothing behind it.
	pub const ASSERTED: Self = Self(0);
	/// Someone it concerns said so, at no cost to them.
	pub const HEARD: Self = Self(1);
	/// Seen in what people did.
	pub const OBSERVED: Self = Self(2);
	/// Someone gave time, reputation or money to act on it.
	pub const COMMITTED: Self = Self(3);

	/// Validates `evidence` as 0 to 3.
	pub fn new(evidence: u8) -> Result<Self> {
		match evidence {
			0..=3 => Self(evidence).xok(),
			_ => bevybail!("evidence {evidence} is not 0 to 3"),
		}
	}

	/// The rung as a number.
	pub fn get(&self) -> u8 { self.0 }

	/// The rung's word, ie `heard`.
	pub fn word(&self) -> &'static str {
		match self.0 {
			0 => "asserted",
			1 => "heard",
			2 => "observed",
			_ => "committed",
		}
	}
}

impl TryFrom<u8> for Evidence {
	type Error = BevyError;
	fn try_from(evidence: u8) -> Result<Self> { Self::new(evidence) }
}

impl From<Evidence> for u8 {
	fn from(evidence: Evidence) -> u8 { evidence.0 }
}

impl core::fmt::Display for Evidence {
	fn fmt(
		&self,
		formatter: &mut core::fmt::Formatter<'_>,
	) -> core::fmt::Result {
		self.0.fmt(formatter)
	}
}

/// What a false claim costs the plan, highest first, so the ordering is the
/// order a sitting picks by.
#[derive(
	Debug,
	Clone,
	Copy,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Reflect,
	Serialize,
	Deserialize,
)]
pub enum Stakes {
	/// The plan fails.
	Fatal,
	/// The plan changes materially.
	Material,
	/// A detail changes.
	Minor,
}

/// Where a [`Claim`] stands, open first, so the ordering is the order a
/// sitting picks by.
#[derive(
	Debug,
	Clone,
	Copy,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Reflect,
	Serialize,
	Deserialize,
)]
pub enum ClaimStatus {
	/// Untested.
	Open,
	/// Its test is assigned and running.
	Testing,
	/// The evidence holds it.
	Held,
	/// The evidence refuted it, or it no longer matters.
	Dropped,
}

impl ClaimStatus {
	/// Whether a claim so placed is still in question, open or testing.
	pub fn in_question(&self) -> bool {
		matches!(self, Self::Open | Self::Testing)
	}
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	fn claim(
		id: &str,
		stakes: Stakes,
		evidence: u8,
		status: ClaimStatus,
	) -> Claim {
		Claim {
			id: id.into(),
			claim: "A claim.".into(),
			block: Address::parse("product#pricing").unwrap(),
			evidence: Evidence::new(evidence).unwrap(),
			stakes,
			test: String::new(),
			status,
			sources: Vec::new(),
		}
	}

	/// The highest stakes first, then the least evidence, then open before
	/// testing, never a claim settled.
	#[beet::test]
	fn picks_by_stakes_then_evidence_then_open() {
		let pick = |claims: &[Claim]| {
			Claim::pick(claims.iter()).map(|claim| claim.id.to_string())
		};
		pick(&[
			claim("minor", Stakes::Minor, 0, ClaimStatus::Open),
			claim("fatal", Stakes::Fatal, 2, ClaimStatus::Open),
		])
		.xpect_eq(Some("fatal".into()));
		pick(&[
			claim("observed", Stakes::Fatal, 2, ClaimStatus::Open),
			claim("heard", Stakes::Fatal, 1, ClaimStatus::Open),
		])
		.xpect_eq(Some("heard".into()));
		pick(&[
			claim("testing", Stakes::Fatal, 1, ClaimStatus::Testing),
			claim("open", Stakes::Fatal, 1, ClaimStatus::Open),
		])
		.xpect_eq(Some("open".into()));
		pick(&[claim("held", Stakes::Fatal, 0, ClaimStatus::Held)])
			.xpect_none();
	}

	/// Evidence is 0 to 3 in its stored form too.
	#[beet::test]
	fn evidence_is_zero_to_three() {
		Evidence::new(3).unwrap().word().xpect_eq("committed");
		Evidence::new(4).unwrap_err();
		MediaType::Json
			.deserialize::<Evidence>(b"4")
			.unwrap_err()
			.to_string()
			.xpect_contains("evidence 4 is not 0 to 3");
	}
}
