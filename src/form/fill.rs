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

	/// The path below the build directory a build writes the filled form's
	/// [`CellsReport`] to, beside the form, which the triage reads as built.
	pub fn cells_path(rubric: &RubricRef) -> RelPath {
		RelPath::new(rubric.package())
			.join(format!("{}.cells.md", rubric.rubric()))
	}

	/// Applies every op in order to the document under `root`, a form of any
	/// format read into the world, answering a line per op and a warning for
	/// one that matched nothing. A cell the form lacks, a locked cell or a
	/// formula fails the build.
	pub fn apply(
		&self,
		world: &mut World,
		root: Entity,
	) -> Result<Vec<String>> {
		let mut log = Vec::new();
		for (index, op) in self.ops.iter().enumerate() {
			let warn = |log: &mut Vec<String>, count: usize, what: &str| {
				if count == 0 {
					log.push(format!("  WARNING op {}: {what}", index + 1));
				}
			};
			match op {
				FillOp::Set { cell, text } => {
					let address = cell.address()?;
					let written = SetText {
						cell: address.clone(),
						text: text.clone(),
					}
					.apply_to(world, root)?;
					log.push(match address {
						CellAddress::Sheet(_) => {
							let written = world
								.entity(written)
								.get::<SheetCellAddress>()
								.map(ToString::to_string)
								.unwrap_or_else(|| cell.to_string());
							format!("set {written} = {text}")
						}
						CellAddress::Table(_) => format!("set {cell}"),
					});
				}
				FillOp::Append { cell, text } => {
					AppendText {
						cell: cell.address()?,
						text: text.clone(),
					}
					.apply_to(world, root)?;
					log.push(format!("append {cell}"));
				}
				FillOp::Check { label } => {
					let count = CheckBox {
						label: label.to_string(),
					}
					.apply_to(world, root)?;
					log.push(format!("check \"{label}\": {count}"));
					warn(&mut log, count, "no checkbox carries that label");
				}
				FillOp::Delete { text } => {
					let count = RemoveParagraphs { text: text.clone() }
						.apply_to(world, root)?;
					log.push(format!(
						"delete \"{text}\": {count} paragraph(s)"
					));
					warn(&mut log, count, "no paragraph carries that text");
				}
				FillOp::Replace { old, new } => {
					let count = ReplaceText {
						old: old.to_string(),
						new: new.to_string(),
					}
					.apply_to(world, root)?;
					log.push(format!("replace \"{old}\": {count}"));
					warn(&mut log, count, "no paragraph carries that text");
				}
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
		label: SmolStr,
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
		old: SmolStr,
		/// The replacement.
		new: SmolStr,
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
	/// A cell of a form, as the blank form's cells name it: a table cell, a
	/// [`TableCellAddress`] `t<table>r<row>c<cell>` counted from 1 in
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
	/// The cell this names, a table cell or a workbook cell.
	pub fn address(&self) -> Result<CellAddress> { CellAddress::parse(&self.0) }
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	#[beet::test]
	fn parses_cells() {
		CellRef::parse("t3r2c1")
			.unwrap()
			.address()
			.unwrap()
			.to_string()
			.xpect_eq("t3r2c1");
		CellRef::parse("Start Here!D3")
			.unwrap()
			.address()
			.unwrap()
			.xpect_eq(CellAddress::Sheet(
				SheetCellAddress::parse("Start Here!D3").unwrap(),
			));
		for bad in ["t0r1c1", "t1r1", "Sheet!3", "!A1", "Sheet!A01", "A1"] {
			CellRef::parse(bad).xpect_err();
		}
	}

	/// One fill spec fills a markdown form and a Word form alike: the same
	/// cells hold the same words after, and the same box is checked.
	#[beet::test]
	fn fills_a_markdown_form_and_a_word_form() {
		let spec = FillSpec {
			ops: vec![
				FillOp::Set {
					cell: CellRef::parse("t1r1c2").unwrap(),
					text: "Acme Stalls".into(),
				},
				FillOp::Check {
					label: "Surveys".into(),
				},
				FillOp::Delete {
					text: "Please delete this sentence".into(),
				},
				FillOp::Replace {
					old: "(Insert name)".into(),
					new: "Acme".into(),
				},
			],
		};
		let markdown = MediaBytes::new_markdown(
			"| Business Name |  |\n|---|---|\n\n- [ ] Surveys\n\n\
			 (Please delete this sentence once completed)\n\n\
			 The business is <mark>(Insert name)</mark>.\n",
		);
		let word = OoxmlFile::word(
			"<w:tbl><w:tr><w:tc><w:p><w:r><w:t>Business Name</w:t></w:r></w:p></w:tc><w:tc><w:p/></w:tc></w:tr></w:tbl>\
			 <w:p><w:sdt><w:sdtPr><w14:checkbox><w14:checked w14:val=\"0\"/></w14:checkbox></w:sdtPr>\
			 <w:sdtContent><w:r><w:t>\u{2610}</w:t></w:r></w:sdtContent></w:sdt><w:r><w:t xml:space=\"preserve\"> Surveys</w:t></w:r></w:p>\
			 <w:p><w:r><w:t>(Please delete this sentence once completed)</w:t></w:r></w:p>\
			 <w:p><w:r><w:t xml:space=\"preserve\">The business is </w:t></w:r>\
			 <w:r><w:rPr><w:highlight w:val=\"yellow\"/></w:rPr><w:t>(Insert name)</w:t></w:r><w:r><w:t>.</w:t></w:r></w:p>",
		)
		.unwrap();
		let filled = [markdown, word].map(|form| {
			let mut world = (TemplatePlugin, DocumentPlugin).into_world();
			let root = world.spawn_empty().id();
			MediaParser::new()
				.parse(ParseContext::new(&mut world.entity_mut(root), &form))
				.unwrap();
			spec.apply(&mut world, root).unwrap().xpect_eq(vec![
				"set t1r1c2".to_string(),
				"check \"Surveys\": 1".into(),
				"delete \"Please delete this sentence\": 1 paragraph(s)".into(),
				"replace \"(Insert name)\": 1".into(),
			]);
			let cells = CellText::listing(&mut world, root)
				.into_iter()
				.map(|cell| cell.to_string())
				.collect::<Vec<_>>();
			let text =
				world.with_state::<ReaderText, _>(|text| text.text(root));
			(cells, text)
		});
		filled[0].0.clone().xpect_eq(filled[1].0.clone());
		for (_, text) in &filled {
			text.clone()
				.xpect_contains("[x] Surveys")
				.xpect_contains("The business is Acme.")
				.xnot()
				.xpect_contains("Please delete");
		}
	}
}
