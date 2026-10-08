use crate::prelude::*;
use beet::prelude::*;

/// Request params for [`EvalCells`], surfaced in `--help`.
#[derive(Reflect)]
struct CellsParams {
	/// The reader rubric whose blank form is dumped, `<package>/<rubric>`,
	/// the path after `cells`.
	rubric: RubricRef,
}

/// `eval/cells <package>/<rubric>`: the cells of a rubric's blank form, the
/// map a fill spec is written against: every table cell and every unlocked
/// workbook cell with what a reader reads in it, a table for a markup
/// `Accept` and the rows for a serde one. Listed from the form on demand and
/// never stored, so it cannot drift from the file.
#[action(route = "cells/*rubric")]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
#[require(ParamsPartial = ParamsPartial::new::<CellsParams>())]
pub async fn EvalCells(
	cx: ActionContext<Request>,
) -> Result<DataPage<Vec<CellText>>> {
	let params = cx.input.parse_params::<CellsParams>()?;
	let workspace = LoadedWorkspace::of(&cx.caller).await?;
	workspace.require_clean()?;
	let (package, spec) = FormSource::of(&workspace, &params.rubric)?;
	let blank = package.store.blob(spec.form.clone()).get_media().await?;
	let cells = FormSource::read(&cx.caller, blank, |world, root| {
		world
			.with_state::<TableCells, _>(|cells| cells.listing(root))
			.xok()
	})
	.await?;
	DataPage::new(&cx.caller, CellText::table(&cells), cells).await
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
				"| Cell | Text |\n|---|---|\n| t1r1c1 | Business Name |\n| t1r1c2 | (empty) |\n\
				 | t2r1c1 | Briefly describe your business: / (Please delete this sentence once completed) |\n",
			);
	}

	/// The same form read as text through beet's store view, its tables
	/// numbered as its cells listing numbers them and its red instruction kept.
	#[beet::test]
	async fn views_the_blank_form_as_markdown() {
		let request = Request::from_cli_str(&format!("view {}", Fixture::FORM))
			.with_header::<header::Accept>(vec![MediaType::Markdown]);
		Fixture::new()
			.await
			.answer(request)
			.await
			.1
			.xpect_contains("<!-- t1 -->\n\n| Business Name |  |\n|---|---|\n")
			.xpect_contains(
				"Briefly describe your business:<br><span style=\"color: #FF0000\">(Please delete this sentence once completed)</span>",
			);
	}
}
