//! The verbs: each a route of `beet-eval` taking its flags in its signature,
//! so `--help` documents them and a strict server refuses any other,
//! resolving the workspace through the store above it. `README.md` lists
//! them; `<EvalRoutes/>` mounts them all.
//!
//! A report verb answers a scene of what it computed, a `#[template]` of HTML
//! nodes rendered as the request accepts, markdown for an agent, ANSI in a
//! terminal or HTML in a browser, and for a serde `Accept`, ie
//! `--accept=application/json`, the stored form of the same value, so an
//! agent reads what a person does. A file a verb writes, a projection or a
//! build's cells, is the render of its scene.
mod answers;
mod blocks;
mod build;
mod cells;
mod check;
mod deck;
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
mod route_call;
mod rows;
mod run;
mod worksheet;
pub use answers::*;
pub use blocks::*;
pub use build::*;
pub use cells::*;
pub use check::*;
pub use deck::*;
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
pub(crate) use route_call::*;
pub use rows::*;
pub(crate) use run::*;
pub use worksheet::*;

use beet::exports::bevy::reflect::enums::Enum;
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
		EvalAnswers,
		EvalApprove,
		EvalCells,
		EvalDeck,
		EvalRows,
		EvalPut,
		EvalDrop,
	]
}

/// A verb's answer when it found something wrong: the text, and a status a
/// process exits 1 on.
fn refusal(text: impl Into<String>) -> Response {
	Response::status_text(StatusCode::UNPROCESSABLE_CONTENT, text.into())
}

/// `media` parsed into the caller's world and `func` run on its root, which
/// is despawned after, whatever `func` answers.
pub(crate) async fn parsed_with<O: 'static + Send + Sync>(
	caller: &AsyncEntity,
	media: MediaBytes,
	func: impl 'static + Send + FnOnce(&mut World, Entity) -> Result<O>,
) -> Result<O> {
	caller
		.world()
		.with(move |world: &mut World| -> Result<O> {
			let root = world.spawn_empty().id();
			let read = ParseContext::new(&mut world.entity_mut(root), &media)
				.parse()
				.map_err(BevyError::from)
				.and_then(|_| func(world, root));
			world.entity_mut(root).despawn();
			read
		})
		.await
}

/// A scene rendered as markdown, the answer to a request accepting it, for a
/// file a verb writes, led by the frontmatter its root's metadata declares
/// in the format `frontmatter` names.
pub(crate) async fn markdown_of(
	caller: &AsyncEntity,
	scene: impl 'static + Send + Sync + Bundle,
	frontmatter: Option<MediaType>,
) -> Result<String> {
	caller
		.world()
		.with(move |world: &mut World| -> Result<String> {
			let entity =
				world.spawn_template(Snippet::from_bundle(scene))?.id();
			let request =
				RequestParts::get("/").with_accept(MediaType::Markdown);
			// a format is named by its variant, as `--frontmatter=toml` is read
			let request = match &frontmatter {
				Some(format) => {
					request.with_param("frontmatter", format.variant_name())
				}
				None => request,
			};
			let text = MarkdownRenderer::new()
				.render(&mut RenderContext::new(world, entity, &request))
				.map(|bytes| bytes.to_string());
			world.entity_mut(entity).despawn();
			text?.xok()
		})
		.await
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	/// Every route reached with each of its own flags and the server's
	/// `--accept`, none refused as a bad request, and a misspelt flag refused
	/// on each, as the strict cli server refuses it.
	#[beet::test]
	async fn every_route_takes_its_flags() {
		let commands = [
			"eval/check --unused",
			"eval/results",
			"eval/next",
			"eval/blocks price-list",
			"eval/worksheet product",
			"eval/project acme_course/01-business-plan",
			"eval/new Acme Ada",
			"eval/grade --eval=product.alternatives-considered --level=2 --anchor=product#idea --by=x --evidence=We\\ weighed\\ a\\ mobile\\ service",
			"eval/build acme_course/01-business-plan --fresh",
			"eval/answers acme_course/01-business-plan",
			"eval/approve acme_course/01-business-plan --by=ada",
			"eval/cells acme_course/01-business-plan",
			"eval/deck packages/acme/assets/week-1.pptx",
			"eval/rows --package=acme_biz --table=evals --key=product.origin",
			"eval/put --package=acme_biz --table=evals --from=packages/acme_biz/evals/product.origin --row=",
			"eval/drop --package=acme_course --table=render --key=01-business-plan",
			"check/document --document=brand",
			"check/frontmatter --keys=created --documents=brand",
			"check/h1 --document=brand",
			"check/tagline --document=index",
			"check/summary --documents=brand",
			"check/agrees --document=index --part=title --section=brand#name-and-tagline",
			"check/asks",
			"check/sections --document=brand --headings=Voice",
			"check/block --name=price-list --section=product#pricing --columns=line --columns=price_ex_gst:num",
			"view packages/acme_course/assets/forms/01-business-plan.docx",
		];
		let mut fixture = Fixture::new().await;
		let mut reached = Vec::new();
		for command in commands {
			let request = Fixture::request(command);
			let route = fixture
				.route_of(&request)
				.unwrap_or_else(|| panic!("`{command}` reaches no route"));
			reached.push(route.entity);
			// a route's own flags, its path's captures and the render flags
			// every report verb inherits from its answer aside
			let inherited = ParamMeta::of::<RenderParams>();
			for param in route.params.own().filter(|param| {
				!inherited.contains(param)
					&& !route.path.iter().any(|segment| {
						!segment.is_static() && segment.name() == param.name()
					})
			}) {
				if !request.has_param(param.name()) {
					panic!("`{command}` leaves out `--{}`", param.name());
				}
			}
			for accept in ["text/markdown", "application/json"] {
				let request =
					Fixture::request(&format!("{command} --accept={accept}"));
				fixture
					.exchange(request)
					.await
					.status()
					.xpect_not_eq(StatusCode::BAD_REQUEST);
			}
			fixture
				.refused(&format!("{command} --bogus"))
				.await
				.xpect_contains("unknown param `--bogus`");
		}
		for route in fixture.routes() {
			reached.contains(&route.entity).xpect_true();
		}
	}
}
