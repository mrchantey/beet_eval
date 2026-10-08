//! The verbs: each a route of `beet-eval` whose flags are its params type, so
//! `--help` documents them, resolving the workspace through the store above
//! it. `README.md` lists them; `<EvalRoutes/>` mounts them all.
//!
//! A report verb answers a scene of what it computed, a `#[template]` of HTML
//! nodes rendered as the request accepts, markdown for an agent, ANSI in a
//! terminal or HTML in a browser, and for a serde `Accept`, ie
//! `--accept=application/json`, the stored form of the same value, so an
//! agent reads what a person does. A file a verb writes, a projection or a
//! build's cells, is the render of its scene.
mod blocks;
mod build;
mod cells;
mod check;
mod drop;
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
pub use drop::*;
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
		EvalDrop,
	]
}

/// A verb's answer when it found something wrong: the text, and a status a
/// process exits 1 on.
fn refusal(text: impl Into<String>) -> Response {
	Response::status_text(StatusCode::UNPROCESSABLE_CONTENT, text.into())
}

/// A scene rendered as markdown, for a file a verb writes.
pub(crate) async fn markdown_of(
	caller: &AsyncEntity,
	scene: impl 'static + Send + Sync + Bundle,
) -> Result<String> {
	caller
		.world()
		.with(move |world: &mut World| -> Result<String> {
			let entity =
				world.spawn_template(Snippet::from_bundle(scene))?.id();
			let text = MarkdownRenderer::new()
				.render(&mut RenderContext::new(entity, world))
				.map(|bytes| bytes.to_string());
			world.entity_mut(entity).despawn();
			text?.xok()
		})
		.await
}
