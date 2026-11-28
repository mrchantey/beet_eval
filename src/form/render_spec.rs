use beet::prelude::*;

/// How one of a reader suite's forms is rendered: a row of the suite's
/// `render` table, keyed on the rubric whose structural lines verify it.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct RenderSpec {
	/// The rubric's id in the same suite, ie `01-business-plan`.
	pub rubric: SmolStr,
	/// The reader's own blank form inside the suite, a Word file or a
	/// workbook, never written to.
	pub form: RelPath,
	/// The built form's file name under `dist/<suite>/`, ie
	/// `01-business-plan.docx`.
	pub output: SmolStr,
	/// What the form needs said beyond its cells: its quirks, what a build
	/// must edit besides cell text, and how the result is read.
	pub notes: String,
}

/// A row of a reader suite's `render` table, keyed on its rubric.
impl TableStoreRow for RenderSpec {
	fn table_name() -> SmolStr { "render".into() }
	fn key(&self) -> TableKey { self.rubric.as_str().into() }
}
