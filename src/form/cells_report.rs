use crate::prelude::*;
use beet::prelude::*;

/// A built form's cells as the build keeps them beside it, to read against
/// the structural lines of its rubric: every addressed cell with what a
/// reader reads in it.
#[template]
pub fn CellsReport(
	/// The built form, ie `dist/sarina_russo/01-business-plan.docx`.
	output: String,
	/// The rubric whose form it is.
	#[prop(required)]
	rubric: RubricRef,
	/// The day it was built.
	#[prop(required)]
	date: Date,
	/// Its cells.
	cells: Vec<CellText>,
) -> impl Bundle {
	rsx! {
		<h1>{format!("Cells of {output}")}</h1>
		<p>
			"Dumped by "<code>{format!("eval/build {rubric}")}</code>{format!(" on {date}. Read these against the structural lines of the rubric ")}
			<code>{rubric.to_string()}</code>"."
		</p>
		{CellText::table(&cells)}
	}
}
