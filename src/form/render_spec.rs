use beet::prelude::*;

/// How one of a reader package's forms is rendered: a row of the package's
/// `render` table, keyed on the rubric whose structural lines verify it.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct RenderSpec {
	/// The rubric's id in the same package, ie `01-business-plan`.
	pub rubric: SmolStr,
	/// The reader's own blank form inside the package, a Word file or a
	/// workbook, never written to.
	pub form: RelPath,
	/// The filled copy's file name under `dist/<package>/`, ie
	/// `01-business-plan.docx`.
	pub output: SmolStr,
	/// What the form needs said beyond its cells: its quirks, what a fill must
	/// do besides set text, and how the result is read.
	pub notes: String,
}

/// A row of a reader package's `render` table, keyed on its rubric.
impl TableStoreRow for RenderSpec {
	fn table_name() -> SmolStr { "render".into() }
	fn key(&self) -> TableKey { self.rubric.as_str().into() }
}
