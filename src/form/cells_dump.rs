use crate::prelude::*;
use beet::prelude::*;

/// The two kinds of blank form a reader package renders, by media type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormKind {
	/// A Word file, `.docx`, filled by table cell.
	Word,
	/// A workbook, `.xlsx`, filled by unlocked cell.
	Workbook,
}

impl FormKind {
	/// The kind of the form at `path`.
	pub fn of(path: &RelPath) -> Result<Self> {
		match MediaType::from_path(path.as_str()) {
			MediaType::Docx => Self::Word,
			MediaType::Xlsx => Self::Workbook,
			_ => bevybail!(
				"`{path}` is no form a build fills: expected a .docx or an .xlsx"
			),
		}
		.xok()
	}
}

/// A form's cells as a builder reads them: every table cell of a Word file,
/// every unlocked cell of a workbook, one `| cell | text |` row each.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellsDump {
	/// The form's kind, which names the table's second column.
	kind: FormKind,
	/// The rows, each `| cell | text |`.
	rows: Vec<String>,
}

impl CellsDump {
	/// The dump's path below the workspace's build directory, beside the
	/// form it describes.
	pub fn path(rubric: &RubricRef) -> RelPath {
		RelPath::new(rubric.package())
			.join(format!("{}.cells.md", rubric.rubric()))
	}

	/// Every table cell of a Word form.
	pub fn word(form: &WordDocument) -> Self {
		Self {
			kind: FormKind::Word,
			rows: form.cells().iter().map(ToString::to_string).collect(),
		}
	}

	/// Every unlocked cell of a workbook form.
	pub fn workbook(form: &Workbook) -> Result<Self> {
		Self {
			kind: FormKind::Workbook,
			rows: form
				.unlocked_cells()?
				.iter()
				.map(ToString::to_string)
				.collect(),
		}
		.xok()
	}

	/// How many cells it dumps.
	pub fn len(&self) -> usize { self.rows.len() }

	/// Whether it dumps nothing.
	pub fn is_empty(&self) -> bool { self.rows.is_empty() }

	/// The rows alone, one per line.
	pub fn rows(&self) -> String {
		self.rows.iter().map(|row| format!("{row}\n")).collect()
	}

	/// The dump as the build writes it beside `output`, the built form.
	pub fn to_markdown(&self, output: &str, rubric: &RubricRef) -> String {
		let head = match self.kind {
			FormKind::Word => "| Cell | Text |",
			FormKind::Workbook => "| Cell | Value |",
		};
		format!(
			"# Cells of {output}\n\nDumped by `eval/build {rubric}` on {}. Read these against \
			 the structural lines of the rubric `{rubric}`.\n\n{head}\n|---|---|\n{}",
			Date::today(),
			self.rows()
		)
	}
}
