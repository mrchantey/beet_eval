use crate::prelude::*;
use beet::prelude::*;

/// One thing the subject's plan rests on that might be false: a row of the
/// workspace package's `claims` register, keyed on its id.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct Claim {
	/// A slug unique in the register, ie `buyers-pay-monthly`.
	pub id: SmolStr,
	/// The claim, one sentence that could turn out false.
	pub claim: String,
	/// The section of the documents that rests on it.
	pub block: Address,
	/// How far it has been tested, 0 to 3: asserted with nothing behind it,
	/// heard from someone it concerns, observed in what people did, or
	/// committed to with money or time.
	pub evidence: u8,
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

/// A row of the workspace package's `claims` register, keyed on its id.
impl TableStoreRow for Claim {
	fn table_name() -> SmolStr { "claims".into() }
	fn key(&self) -> TableKey { self.id.as_str().into() }
}

/// What a false claim costs the plan.
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

/// Where a [`Claim`] stands.
#[derive(
	Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize,
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
