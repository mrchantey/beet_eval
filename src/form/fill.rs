use crate::prelude::*;
use crate::text_type::text_type;
use beet::prelude::*;

/// What a builder writes for one form: the operations that turn a copy of the
/// reader's blank form into the filled one, applied in order by `eval/build`.
/// Kept at `dist/<package>/<rubric>.fill.json` beside the form it fills.
#[derive(Debug, Default, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct FillSpec {
	/// The operations, in order.
	pub ops: Vec<FillOp>,
}

impl FillSpec {
	/// The spec's path below the workspace's build directory.
	pub fn path(rubric: &RubricRef) -> RelPath {
		RelPath::new(rubric.package())
			.join(format!("{}.fill.json", rubric.rubric()))
	}

	/// Applies every op to a Word form in order, answering a line per op and
	/// a warning for one that matched nothing.
	pub fn apply_word(&self, form: &mut WordDocument) -> Result<Vec<String>> {
		let mut log = Vec::new();
		for (index, op) in self.ops.iter().enumerate() {
			let warn = |log: &mut Vec<String>, count: usize, what: &str| {
				if count == 0 {
					log.push(format!("  WARNING op {}: {what}", index + 1));
				}
			};
			match op {
				FillOp::Set { cell, text } => {
					form.set_cell(cell.table_address()?, text)?;
					log.push(format!("set {cell}"));
				}
				FillOp::Append { cell, text } => {
					form.append_cell(cell.table_address()?, text)?;
					log.push(format!("append {cell}"));
				}
				FillOp::Check { label } => {
					let count = form.check(label)?;
					log.push(format!("check \"{label}\": {count}"));
					warn(&mut log, count, "no checkbox carries that label");
				}
				FillOp::Delete { text } => {
					let count = form.delete_paragraphs(text)?;
					log.push(format!(
						"delete \"{text}\": {count} paragraph(s)"
					));
					warn(&mut log, count, "no paragraph carries that text");
				}
				FillOp::Replace { old, new } => {
					let count = form.replace_text(old, new)?;
					log.push(format!("replace \"{old}\": {count}"));
					warn(&mut log, count, "no paragraph carries that text");
				}
			}
		}
		log.xok()
	}

	/// Applies every op to a workbook form in order: only `Set` applies, a
	/// locked cell or a formula failing the build, and any other op is
	/// skipped with a warning.
	pub fn apply_workbook(&self, form: &mut Workbook) -> Result<Vec<String>> {
		let mut log = Vec::new();
		for (index, op) in self.ops.iter().enumerate() {
			match op {
				FillOp::Set { cell, text } => {
					let written = form.set(&cell.sheet_address()?, text)?;
					log.push(format!("set {written} = {text}"));
				}
				other => log.push(format!(
					"WARNING op {}: only `Set` applies to a workbook, skipped `{}`",
					index + 1,
					other.word()
				)),
			}
		}
		log.xok()
	}
}

/// One operation of a [`FillSpec`]. Text is plain, a newline starting a new
/// paragraph.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub enum FillOp {
	/// Replaces a Word cell's paragraphs with one per line, in the cell's own
	/// paragraph and run style; writes a workbook value, a number when it
	/// parses as one, into an unlocked cell, refusing a locked one or a
	/// formula.
	Set {
		/// The cell.
		cell: CellRef,
		/// The text or value.
		text: String,
	},
	/// Adds paragraphs after a Word cell's own, for a one-cell box whose prompt
	/// shares the cell with its answer.
	Append {
		/// The cell.
		cell: CellRef,
		/// The text.
		text: String,
	},
	/// Ticks the checkbox control whose paragraph carries the label: glyph
	/// ticked, bold and highlighted.
	Check {
		/// The label beside the box.
		label: String,
	},
	/// Removes every paragraph carrying the text, ie a red instruction
	/// sentence; a cell keeps one empty paragraph.
	Delete {
		/// The text the paragraphs carry.
		text: String,
	},
	/// Replaces text inside the runs that carry it, keeping their formatting,
	/// so a highlighted placeholder stays highlighted.
	Replace {
		/// The text as the file carries it.
		old: String,
		/// The replacement.
		new: String,
	},
}

impl FillOp {
	/// The op's name, ie `Set`.
	pub fn word(&self) -> &'static str {
		match self {
			Self::Set { .. } => "Set",
			Self::Append { .. } => "Append",
			Self::Check { .. } => "Check",
			Self::Delete { .. } => "Delete",
			Self::Replace { .. } => "Replace",
		}
	}
}

text_type!(
	/// A cell of a form, as the blank form's cells dump names it: a Word table
	/// cell, a [`TableCellAddress`] `t<table>r<row>c<cell>` counted from 1 in
	/// document order, ie `t3r2c1`, or a workbook cell, a
	/// [`SheetCellAddress`] `<sheet>!<column><row>`, ie `Start Here!D3`.
	CellRef,
	|text| match TableCellAddress::parse(&text).is_ok()
		|| SheetCellAddress::parse(&text).is_ok()
	{
		true => OK,
		false => bevybail!(
			"`{text}` is not a cell, expected `t<n>r<n>c<n>` or `<sheet>!<A1>`"
		),
	}
);

impl CellRef {
	/// The Word table cell this names, refusing a workbook cell.
	pub fn table_address(&self) -> Result<TableCellAddress> {
		TableCellAddress::parse(&self.0).map_err(|_| {
			bevyhow!("`{self}` is a workbook cell, not a Word table cell")
		})
	}

	/// The workbook cell this names, refusing a Word table cell.
	pub fn sheet_address(&self) -> Result<SheetCellAddress> {
		SheetCellAddress::parse(&self.0).map_err(|_| {
			bevyhow!("`{self}` is a Word table cell, not a workbook cell")
		})
	}
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	#[beet::test]
	fn parses_cells() {
		CellRef::parse("t3r2c1")
			.unwrap()
			.table_address()
			.unwrap()
			.to_string()
			.xpect_eq("t3r2c1");
		CellRef::parse("Start Here!D3")
			.unwrap()
			.sheet_address()
			.unwrap()
			.sheet
			.as_str()
			.xpect_eq("Start Here");
		CellRef::parse("t1r1c1")
			.unwrap()
			.sheet_address()
			.xpect_err();
		for bad in ["t0r1c1", "t1r1", "Sheet!3", "!A1", "Sheet!A01", "A1"] {
			CellRef::parse(bad).xpect_err();
		}
	}
}
