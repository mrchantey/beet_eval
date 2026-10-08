use crate::prelude::*;
use beet::prelude::*;

/// The params of [`BlockCheck`].
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct BlockCheckParams {
	/// The block's name, its fence's info string after `csv`.
	pub name: SmolStr,
	/// Where it lives, ie `product#pricing`.
	pub section: Address,
	/// Its columns in order, each spelled as [`Column`] displays, ie
	/// `price_ex_gst:num`.
	pub columns: Vec<SmolStr>,
}

impl BlockCheckParams {
	/// The most cell problems a failure lists.
	const SHOWN: usize = 4;

	/// The verdict on `documents`.
	pub fn decide(&self, documents: &mut DocumentSet) -> CheckVerdict {
		let home = self.section.document_name();
		let fail = |detail: String| CheckVerdict::fail(detail, [home]);
		let columns = match self
			.columns
			.iter()
			.map(|column| Column::parse(column))
			.collect::<Result<Vec<_>>>()
		{
			Ok(columns) => columns,
			Err(err) => return fail(err.to_string()),
		};
		let found = documents
			.data_blocks()
			.into_iter()
			.filter(|(_, block)| block.name == self.name)
			.collect::<Vec<_>>();
		let at = |document: &DocumentFile, block: &DataBlock| {
			format!("{}:{}", documents.path_of(document), block.line)
		};
		let (document, block) = match found.as_slice() {
			[] => {
				return fail(format!(
					"no block named {} under {}/",
					self.name,
					documents.dir()
				));
			}
			[(document, block)] => (document, block),
			many => {
				return fail(format!(
					"defined {} times: {}",
					many.len(),
					many.iter()
						.map(|(document, block)| at(document, block))
						.collect::<Vec<_>>()
						.join(", ")
				));
			}
		};
		if block.format != BlockFormat::Csv {
			return fail(format!(
				"{} is json, expected csv",
				at(document, block)
			));
		}
		let inside = document.name == home
			|| document
				.name
				.strip_prefix(home)
				.is_some_and(|rest| rest.starts_with('/'));
		if !inside {
			return fail(format!(
				"{} is not in the {home} document",
				at(document, block)
			));
		}
		let own_file = document.name == home;
		if let Some(section) = self.section.section().filter(|_| own_file) {
			if block.section.as_deref() != Some(section) {
				return fail(format!(
					"{} sits under #{}, expected #{section}",
					at(document, block),
					block.section.as_deref().unwrap_or("no section")
				));
			}
		}
		let names = columns
			.iter()
			.map(|column| column.name.as_str())
			.collect::<Vec<_>>()
			.join(",");
		if block.header.join(",") != names {
			let header = match block.header.is_empty() {
				true => "empty".to_string(),
				false => block.header.join(","),
			};
			return fail(format!("columns are {header}, expected {names}"));
		}
		let mut problems = Vec::new();
		for (index, row) in block.rows.iter().enumerate() {
			if row.len() != columns.len() {
				problems.push(format!(
					"row {} has {} cells",
					index + 1,
					row.len()
				));
				continue;
			}
			for (column, cell) in columns.iter().zip(row) {
				if !column.kind.admits(cell) {
					problems.push(format!(
						"row {} {}='{cell}' is not a {}",
						index + 1,
						column.name,
						column.kind.spelling()
					));
				}
			}
		}
		if problems.is_empty() {
			return CheckVerdict::pass(format!(
				"{} rows at {}",
				block.rows.len(),
				at(document, block)
			));
		}
		let more = problems.len().saturating_sub(Self::SHOWN);
		problems.truncate(Self::SHOWN);
		let mut detail = problems.join("; ");
		if more > 0 {
			detail.push_str(&format!("; {more} more"));
		}
		fail(detail)
	}
}

/// `check/block`: passes when the named `csv` block is defined exactly once
/// under the documents directory, in the section's document and, when it sits
/// in that document's own file, under that section, with exactly these
/// columns, every cell of a typed column of its type.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
#[require(
	PathPartial = PathPartial::new("block"),
	ParamsPartial = ParamsPartial::new::<BlockCheckParams>()
)]
pub async fn BlockCheck(cx: ActionContext<Request>) -> Result<Response> {
	DocumentSet::answer_check(&cx, BlockCheckParams::decide).await
}
