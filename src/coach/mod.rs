//! The coach's material and its sitting: the workspace suite's claims
//! register, the claims the subject's plan rests on with how far each has been
//! tested on the [`Evidence`] ladder, and the document suite's coach
//! actions, the moves a sitting makes against them.
//!
//! A sitting is a conversation with the owner, held by `RunThread` in a
//! workspace's `coach.bsx` under a surface to type into and entered with the
//! start request, `--claim=<claim id>`, so it is never a unit the loop runs.
//! It holds the claim [`Claim::pick`] picks, the highest stakes, then the
//! least evidence, then an open one, which `eval/next` names when it is
//! fatal at evidence 0 or 1, [`HoldClaim`] reading it off the entry. The
//! coach asks one question per turn, gives a decision no default, keeps the
//! register through `eval/put`, and ends with one assignment written to the
//! claim's test, which [`AssignmentWritten`] sees and ends the sitting on.
mod assignment_written;
mod claim;
mod coach_action;
pub use assignment_written::*;
pub use claim::*;
pub use coach_action::*;
