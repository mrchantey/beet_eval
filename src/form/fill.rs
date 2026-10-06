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

text_type!(
	/// A cell of a form, as the blank form's cells dump names it: a Word table
	/// cell `t<table>r<row>c<cell>` counted from 1 in document order, ie
	/// `t3r2c1`, or a workbook cell `<sheet>!<column><row>`, ie `Start
	/// Here!D3`.
	CellRef,
	|text| match CellRef::table_parts(&text).is_some()
		|| CellRef::sheet_parts(&text).is_some()
	{
		true => OK,
		false => bevybail!(
			"`{text}` is not a cell, expected `t<n>r<n>c<n>` or `<sheet>!<A1>`"
		),
	}
);

impl CellRef {
	/// A Word table cell's table, row and cell, each from 1.
	pub fn table_cell(&self) -> Option<(u32, u32, u32)> {
		Self::table_parts(&self.0)
	}

	/// A workbook cell's sheet and `A1` address.
	pub fn sheet_cell(&self) -> Option<(&str, &str)> {
		Self::sheet_parts(&self.0)
	}

	fn table_parts(text: &str) -> Option<(u32, u32, u32)> {
		let rest = text.strip_prefix('t')?;
		let (table, rest) = rest.split_once('r')?;
		let (row, cell) = rest.split_once('c')?;
		let number = |part: &str| {
			part.chars()
				.all(|char| char.is_ascii_digit())
				.then(|| part.parse::<u32>().ok())
				.flatten()
				.filter(|number| *number > 0)
		};
		Some((number(table)?, number(row)?, number(cell)?))
	}

	fn sheet_parts(text: &str) -> Option<(&str, &str)> {
		let (sheet, cell) = text.rsplit_once('!')?;
		let letters = cell
			.chars()
			.take_while(|char| char.is_ascii_uppercase())
			.count();
		let (column, row) = cell.split_at(letters);
		(!sheet.is_empty()
			&& !column.is_empty()
			&& !row.is_empty()
			&& !row.starts_with('0')
			&& row.chars().all(|char| char.is_ascii_digit()))
		.then_some((sheet, cell))
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
			.table_cell()
			.xpect_eq(Some((3, 2, 1)));
		CellRef::parse("Start Here!D3")
			.unwrap()
			.sheet_cell()
			.xpect_eq(Some(("Start Here", "D3")));
		for bad in ["t0r1c1", "t1r1", "Sheet!3", "!A1", "Sheet!A01", "A1"] {
			CellRef::parse(bad).xpect_err();
		}
	}
}
