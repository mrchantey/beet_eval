//! The coach's material: the workspace package's claims register, the claims
//! the subject's plan rests on with how far each has been tested, and the
//! document package's coach actions, the moves a sitting makes against them.
//! A sitting picks the claim with the highest stakes and the least evidence,
//! asks one question per turn, and ends with one assignment, written to the
//! register as the claim's test.
mod claim;
mod coach_action;
pub use claim::*;
pub use coach_action::*;
