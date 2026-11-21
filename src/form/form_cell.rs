use crate::text_type::text_type;
use beet::prelude::*;

text_type!(
	/// A cell of a form, named by its place: a table's cell as `t<table>/<A1>`,
	/// the table's place among the document's tables and the cell's grid slot,
	/// ie `t0/B1` for the second cell of the first table's first row, as Word
	/// and Writer name a table's cells in their formulas; a workbook's as
	/// `<sheet>!<A1>`, ie `Start Here!D3`. Tables count from 0 as every
	/// index does; a `/`, which no sheet name may hold, tells the two apart.
	CellRef,
	|text| CellRef::slot_of(&text).map(|_| ())
);

impl CellRef {
	/// The cell at `slot` of the document's table counted `table` from 0.
	pub fn table(table: usize, slot: UVec2) -> Self {
		Self(format!("t{table}/{}", A1Reference(slot)).into())
	}

	/// The cell at `slot` of the worksheet `sheet`.
	pub fn sheet(sheet: &str, slot: UVec2) -> Self {
		Self(format!("{sheet}!{}", A1Reference(slot)).into())
	}

	/// The grid slot `text` names, checking its table or sheet half.
	fn slot_of(text: &str) -> Result<UVec2> {
		let (place, reference) =
			match (text.split_once('/'), text.rsplit_once('!')) {
				(Some((table, reference)), _) => {
					table
						.strip_prefix('t')
						.filter(|index| !index.is_empty())
						.and_then(|index| index.parse::<usize>().ok())
						.ok_or_else(|| {
							bevyhow!("`{text}` names no table, ie t0/B1")
						})?;
					(table, reference)
				}
				(None, Some((sheet, reference))) => (sheet, reference),
				(None, None) => {
					bevybail!("`{text}` is no cell, ie t0/B1 or Sheet1!B1")
				}
			};
		if place.is_empty() {
			bevybail!("`{text}` names no table or sheet");
		}
		A1Reference::parse(reference)?.0.xok()
	}
}

/// One cell of a form as a reader reads it.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct FormCell {
	/// Its place.
	pub cell: CellRef,
	/// Its paragraphs, trimmed, the empty dropped and the rest joined by
	/// ` / `; a formula as its `data-formula`.
	pub text: SmolStr,
}

impl FormCell {
	/// Every cell under `root` a person may type into, in document order.
	pub fn listing(world: &mut World, root: Entity) -> Vec<FormCell> {
		world.with_state::<FormCellQuery, _>(|query| query.listing(root))
	}

	/// The cells as a table headed `Cell` and `Text`, an empty cell's text
	/// `(empty)`.
	pub fn table(cells: &[FormCell]) -> impl Bundle + use<> {
		let rows = cells
			.iter()
			.map(|cell| (SmolStr::new(cell.cell.as_str()), cell.shown_text()))
			.collect::<Vec<_>>();
		let row = |tag: &'static str, first: SmolStr, second: SmolStr| {
			(Element::new("tr"), children![
				(Element::new(tag), children![Value::Str(first)]),
				(Element::new(tag), children![Value::Str(second)]),
			])
		};
		(
			Element::new("table"),
			OnSpawn::new(move |entity| {
				entity.with_children(|table| {
					table.spawn(row("th", "Cell".into(), "Text".into()));
					for (cell, text) in rows {
						table.spawn(row("td", cell, text));
					}
				});
			}),
		)
	}

	/// Its text, `(empty)` when it holds none.
	fn shown_text(&self) -> SmolStr {
		match self.text.is_empty() {
			true => "(empty)".into(),
			false => self.text.clone(),
		}
	}
}

impl core::fmt::Display for FormCell {
	fn fmt(
		&self,
		formatter: &mut core::fmt::Formatter<'_>,
	) -> core::fmt::Result {
		write!(
			formatter,
			"| {} | {} |",
			self.cell.as_str(),
			self.shown_text().replace('|', "\\|")
		)
	}
}

/// The cells of a form read into the world, named by place: a worksheet's
/// table by its `article data-worksheet`, every other table by its place
/// among the document's tables outside a worksheet, nested ones included,
/// in document order.
#[derive(SystemParam)]
pub struct FormCellQuery<'w, 's> {
	grid: TableGrid<'w, 's>,
	text: DocumentText<'w, 's>,
	tree: RenderTreeQuery<'w, 's>,
	views: ElementQuery<'w, 's>,
	elements: Query<'w, 's, &'static Element>,
}

impl FormCellQuery<'_, '_> {
	/// Every cell under `root` a person may type into, in document order.
	pub fn listing(&self, root: Entity) -> Vec<FormCell> {
		let mut tables = 0;
		let mut cells = Vec::new();
		for table in self
			.tree
			.iter_descendants_inclusive_depth_first(root)
			.filter(|entity| self.tag(*entity) == Some("table"))
			.collect::<Vec<_>>()
		{
			let sheet = self.sheet_of(table);
			let name = |slot: UVec2| match &sheet {
				Some(sheet) => CellRef::sheet(sheet, slot),
				None => CellRef::table(tables, slot),
			};
			for cell in self
				.grid
				.cells(table)
				.into_iter()
				.filter(|cell| !self.grid.is_locked(cell.entity))
			{
				cells.push(FormCell {
					cell: name(cell.start),
					text: self.cell_text(cell.entity).into(),
				});
			}
			if sheet.is_none() {
				tables += 1;
			}
		}
		cells
	}

	/// The name of the worksheet `table` is in, if any.
	fn sheet_of(&self, table: Entity) -> Option<String> {
		self.tree
			.iter_ancestors_inclusive(table)
			.find_map(|entity| {
				let view = self.views.get(entity).ok()?;
				(view.tag() == "article")
					.then(|| view.attribute("data-worksheet"))
					.flatten()
					.map(|_| view.attribute_string("data-worksheet"))
			})
	}

	/// What a reader reads in `cell`: its `data-formula`, else its blocks'
	/// text, each trimmed, the empty dropped and the rest joined by ` / `,
	/// and its own text when it holds no block.
	fn cell_text(&self, cell: Entity) -> String {
		if let Some(formula) = self
			.views
			.get(cell)
			.ok()
			.filter(|view| view.attribute("data-formula").is_some())
			.map(|view| view.attribute_string("data-formula"))
		{
			return formula;
		}
		let blocks = self.text.blocks(cell);
		match blocks.is_empty() {
			true => vec![self.text.own_text(cell)],
			false => blocks
				.into_iter()
				.map(|block| self.text.text(block))
				.collect(),
		}
		.iter()
		.map(|text| text.split_whitespace().collect::<Vec<_>>().join(" "))
		.filter(|text| !text.is_empty())
		.collect::<Vec<_>>()
		.join(" / ")
	}

	fn tag(&self, entity: Entity) -> Option<&str> {
		self.elements.get(entity).ok().map(Element::tag)
	}
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	#[beet::test]
	fn names_cells_by_place() {
		CellRef::table(2, UVec2::new(0, 1))
			.as_str()
			.xpect_eq("t2/A2");
		CellRef::sheet("Start Here", UVec2::new(3, 2))
			.as_str()
			.xpect_eq("Start Here!D3");
		for good in ["t0/B1", "Start Here!D3", "a!b!C2"] {
			CellRef::parse(good).unwrap();
		}
		for bad in
			["t/B1", "tx/B1", "t0/1", "Sheet!3", "!A1", "Sheet!A01", "A1"]
		{
			CellRef::parse(bad).xpect_err();
		}
	}

	/// A markdown form and a Word form holding the same tables list the same
	/// cells, a workbook's by sheet, its locked cells left out.
	#[beet::test]
	fn lists_cells_alike_in_every_format() {
		let markdown = MediaBytes::new_markdown(
			"| Business Name |  |\n|---|---|\n\n| Briefly describe it: |\n|---|\n",
		);
		let word = WordFixture::new(
			"<w:tbl><w:tr><w:tc><w:p><w:r><w:t>Business Name</w:t></w:r></w:p></w:tc><w:tc><w:p/></w:tc></w:tr></w:tbl>\
			 <w:p/>\
			 <w:tbl><w:tr><w:tc><w:p><w:r><w:t>Briefly describe it:</w:t></w:r></w:p></w:tc></w:tr></w:tbl>",
		)
		.bytes();
		let [markdown, word] = [markdown, word].map(|form| {
			Fixture::cells_of(&form)
				.iter()
				.map(ToString::to_string)
				.collect::<Vec<_>>()
		});
		markdown.clone().xpect_eq(vec![
			"| t0/A1 | Business Name |".to_string(),
			"| t0/B1 | (empty) |".into(),
			"| t1/A1 | Briefly describe it: |".into(),
		]);
		word.xpect_eq(markdown);
	}

	/// A workbook's cells are named by sheet, its locked cells left out.
	#[beet::test]
	fn lists_a_workbook_by_sheet() {
		let workbook = Fixture::written(
			&MediaBytes::new_html(
				"<article data-worksheet=\"Start Here\"><table><tr>\
				 <td contenteditable=\"false\">Business Name</td><td>Acme</td>\
				 </tr></table></article>",
			),
			MediaType::Xlsx,
		);
		Fixture::cells_of(&workbook)
			.iter()
			.map(ToString::to_string)
			.collect::<Vec<_>>()
			.xpect_eq(vec!["| Start Here!B1 | Acme |".to_string()]);
	}
}
