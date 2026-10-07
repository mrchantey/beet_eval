//! The verbs: each a route of `beet-eval` whose flags are its params type, so
//! `--help` documents them, resolving the workspace through the store above
//! it. `README.md` lists them; `<EvalRoutes/>` mounts them all.
//!
//! A report verb answers in markdown or, with `--format=json`, the stored
//! form of what it computed, so an agent reads the same value a person does.
mod blocks;
mod build;
mod cells;
mod check;
#[cfg(test)]
mod fixture;
mod grade;
mod laws;
mod new;
mod next;
mod project;
mod put;
mod results;
mod run;
mod worksheet;
pub use blocks::*;
pub use build::*;
pub use cells::*;
pub use check::*;
#[cfg(test)]
pub(crate) use fixture::*;
pub use grade::*;
pub(crate) use laws::*;
pub use new::*;
pub use next::*;
pub use project::*;
pub use put::*;
pub use results::*;
pub(crate) use run::*;
pub use worksheet::*;

use beet::prelude::*;

/// `<EvalRoutes/>`: every verb as a child, authored under the `eval` route.
///
/// ```bsx
/// <Route path="eval"><EvalRoutes/></Route>
/// ```
#[template]
pub fn EvalRoutes() -> impl Bundle {
	children![
		EvalCheck,
		EvalResults,
		EvalNext,
		EvalBlocks,
		EvalWorksheet,
		EvalProject,
		EvalNew,
		EvalGrade,
		EvalBuild,
		EvalCells,
		EvalPut,
	]
}

/// How a report verb answers.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum ReportFormat {
	/// Markdown, for a person.
	#[default]
	Md,
	/// The stored form, for a tool.
	Json,
}

/// A verb's answer when it found something wrong: the text, and a status a
/// process exits 1 on.
fn refusal(text: impl Into<String>) -> Response {
	Response::status_text(StatusCode::UNPROCESSABLE_CONTENT, text.into())
}
