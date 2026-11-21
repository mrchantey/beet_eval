use crate::prelude::*;
use beet::prelude::*;

/// `eval/cells <package>/<rubric>`: the cells of a rubric's blank form, the
/// map a builder writes against: every cell a person may type into, named by
/// its place, with what a reader reads in it, a table for a markup `Accept`
/// and the rows for a serde one. Listed from the form on demand and never
/// stored, so it cannot drift from the file.
#[action(route = "cells/*rubric")]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
pub async fn EvalCells(
	/// The reader rubric whose blank form is dumped, `<package>/<rubric>`,
	/// the path after `cells`.
	#[param]
	rubric: RubricRef,
	cx: ActionContext<Request>,
) -> Result<DataPage<Vec<FormCell>>> {
	let workspace = LoadedWorkspace::of(&cx.caller).await?;
	workspace.require_clean()?;
	let (package, spec) = FormSource::of(&workspace, &rubric)?;
	let blank = package.store.blob(spec.form.clone()).get_media().await?;
	let cells = parsed_with(&cx.caller, blank, |world, root| {
		FormCell::listing(world, root).xok()
	})
	.await?;
	DataPage::new(&cx.caller, FormCell::table(&cells), cells).await
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	#[beet::test]
	async fn lists_the_blank_form() {
		Fixture::new()
			.await
			.ok("eval/cells acme_course/01-business-plan")
			.await
			.xpect_eq(
				"| Cell | Text |\n|---|---|\n| t0/A1 | Business Name |\n| t0/B1 | (empty) |\n\
				 | t1/A1 | Briefly describe your business: / (Please delete this sentence once completed) |\n",
			);
	}

	/// The same form read as text through beet's store view: its tables and
	/// its checkbox, the red instruction's colour no part of markdown.
	#[beet::test]
	async fn views_the_blank_form_as_markdown() {
		let request = Request::from_cli_str(&format!("view {}", Fixture::FORM))
			.with_header::<header::Accept>(vec![MediaType::Markdown]);
		Fixture::new()
			.await
			.answer(request)
			.await
			.1
			.xpect_contains("| Business Name |  |\n|---|---|\n")
			.xpect_contains("(Please delete this sentence once completed)")
			.xpect_contains("[ ] Surveys");
	}
}
