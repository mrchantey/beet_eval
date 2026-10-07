use super::*;
use crate::prelude::*;
use beet::prelude::*;

/// Request params for [`EvalCells`], surfaced in `--help`.
#[derive(Reflect)]
struct CellsParams {
	/// The reader rubric whose blank form is dumped, `<package>/<rubric>`,
	/// the path after `cells`.
	rubric: Vec<String>,
}

/// `eval/cells <package>/<rubric>`: the cells of a rubric's blank form, one
/// `| cell | text |` row each, the map a fill spec is written against. Dumped
/// from the form on demand and never stored, so it cannot drift from the
/// file.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
#[require(
	PathPartial = PathPartial::new("cells/*rubric"),
	ParamsPartial = ParamsPartial::new::<CellsParams>()
)]
pub async fn EvalCells(cx: ActionContext<Request>) -> Result<Response> {
	let params = cx.input.parse_params::<CellsParams>()?;
	let workspace = LoadedWorkspace::of(&cx.caller).await?;
	workspace.require_clean()?;
	let reference = RubricRef::parse(params.rubric.join("/"))?;
	let (package, spec) = FormSource::of(&workspace, &reference)?;
	let blank = package.store.get(&spec.form).await?;
	let cells = match FormKind::of(&spec.form)? {
		FormKind::Word => {
			CellsDump::word(&WordDocument::from_bytes(blank.to_vec())?)
		}
		FormKind::Workbook => {
			CellsDump::workbook(&Workbook::from_bytes(blank.to_vec())?)?
		}
	};
	Response::ok_text(cells.rows()).xok()
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	#[beet::test]
	async fn dumps_the_blank_form() {
		Fixture::new()
			.await
			.ok("eval/cells acme_course/01-business-plan")
			.await
			.xpect_eq(
				"| t1r1c1 | Business Name |\n| t1r1c2 | (empty) |\n\
				 | t2r1c1 | Briefly describe your business: / (Please delete this sentence once completed) |\n",
			);
	}

	/// The same form read as text through beet's store view, its tables
	/// numbered as the cells dump numbers them and its red instruction kept.
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
