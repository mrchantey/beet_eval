//! Evaluates a subject against packages of evals and rubrics: checks, grades, results and the next unit of work, with markdown documents as the first subject and Office forms as the renderer
// the harness main for `cargo test --lib`; cfg gated so a plain build does not
// need the facade's `testing` feature
#[cfg(test)]
beet::test_main!();

mod plugin;

/// Exports the most commonly used items.
pub mod prelude {
	pub use crate::plugin::*;
}
