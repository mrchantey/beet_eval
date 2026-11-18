//! The coach's material and its sitting: the workspace package's claims
//! register, the claims the subject's plan rests on with how far each has been
//! tested on the [`Evidence`] ladder, and the document package's coach
//! actions, the moves a sitting makes against them.
//!
//! A sitting is a route, `coach/:claim`, building a thread from its template
//! per request. It holds the claim [`Claim::pick`] picks, the highest stakes,
//! then the least evidence, then an open one, which `eval/next` names when it
//! is fatal at evidence 0 or 1. The coach asks one question per turn, gives a
//! decision no default, keeps the register through `eval/put`, and ends with
//! one assignment written to the claim's test, which [`AssignmentWritten`]
//! sees and ends the sitting on.
mod assignment_written;
mod claim;
mod coach_action;
pub use assignment_written::*;
pub use claim::*;
pub use coach_action::*;
