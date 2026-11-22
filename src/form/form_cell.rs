use crate::text_type::text_type;
use beet::prelude::*;

text_type!(
	/// A cell of a form, named by its place: `<table>/<x>,<y>`, its table
	/// then its slot in that table's grid, both counted from 0 as every index
	/// in beet is, `x` the column and `y` the row. A table outside a
	/// worksheet is `t<n>` by its place among the document's tables, ie
	/// `t0/1,0` for the second cell of the first table's first row; a
	/// worksheet's is its name, ie `Start Here/3,2`, which a `/` never
	/// appears in.
	CellRef,
	|text| CellRef::slot_of(&text).map(|_| ())
);

impl CellRef {
	/// The cell at `slot` of the document's table counted `table` from 0.
	pub fn table(table: usize, slot: UVec2) -> Self {
		Self::at(&format!("t{table}"), slot)
	}

	/// The cell at `slot` of the worksheet `sheet`.
	pub fn sheet(sheet: &str, slot: UVec2) -> Self { Self::at(sheet, slot) }

	fn at(place: &str, slot: UVec2) -> Self {
		Self(format!("{place}/{},{}", slot.x, slot.y).into())
	}

	/// The grid slot `text` names, checking it names a table or sheet.
	fn slot_of(text: &str) -> Result<UVec2> {
		let (place, slot) = text
			.rsplit_once('/')
			.filter(|(place, _)| !place.is_empty())
			.ok_or_else(|| bevyhow!("`{text}` is no cell, ie t0/1,0"))?;
		if place.contains('/') {
			bevybail!("`{text}` names no table or sheet, ie t0/1,0");
		}
		slot.split_once(',')
			.and_then(|(x, y)| {
				let parse = |index: &str| {
					index
						.chars()
						.all(|char| char.is_ascii_digit())
						.then(|| index.parse::<u32>().ok())
						.flatten()
				};
				UVec2::new(parse(x)?, parse(y)?).xsome()
			})
			.ok_or_else(|| {
				bevyhow!("`{text}` names no slot, ie t0/1,0, column then row")
			})
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
			.xpect_eq("t2/0,1");
		CellRef::sheet("Start Here", UVec2::new(3, 2))
			.as_str()
			.xpect_eq("Start Here/3,2");
		for good in ["t0/1,0", "Start Here/3,2", "a!b/0,12"] {
			CellRef::parse(good).unwrap();
		}
		for bad in ["t0/B1", "t0/1", "/1,0", "a/b/1,0", "t0/1,-2", "t0/+1,0"] {
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
			"| t0/0,0 | Business Name |".to_string(),
			"| t0/1,0 | (empty) |".into(),
			"| t1/0,0 | Briefly describe it: |".into(),
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
			.xpect_eq(vec!["| Start Here/1,0 | Acme |".to_string()]);
	}
}
