use crate::prelude::*;
use beet::prelude::*;

/// A form's answers as markdown, written and reviewed before any of the
/// form's own XML is edited: one `## <cell>` section per cell answered, named
/// by its [`CellRef`], its body the answer as markdown, and one `## Edits`
/// section listing what the form needs beyond its cells' text, one item each,
/// ie a box ticked or a red sentence removed. Anything before the first
/// section, ie a title, is the sheet's own.
///
/// ```md
/// ## t3/0,0
///
/// Acme Stalls sells handmade market stalls.
///
/// ## Edits
///
/// - Tick "Sole trader" in 2.1.
/// ```
#[derive(Debug, Default, Clone, PartialEq)]
pub struct AnswerSheet {
	/// The cells answered, in the sheet's order.
	pub answers: Vec<CellAnswer>,
	/// The edits beyond the cells' text, in the sheet's order.
	pub edits: Vec<String>,
}

/// One cell's answer in an [`AnswerSheet`].
#[derive(Debug, Clone, PartialEq)]
pub struct CellAnswer {
	/// The cell.
	pub cell: CellRef,
	/// What it is to hold, as markdown.
	pub answer: String,
}

impl AnswerSheet {
	/// The heading of the section listing the edits.
	pub const EDITS: &str = "Edits";

	/// The sheet `text` writes, refusing a heading that names no cell and a
	/// cell answered twice.
	pub fn parse(text: &str) -> Result<Self> {
		let mut sheet = Self::default();
		for (heading, body) in Self::sections(text) {
			if heading == Self::EDITS {
				sheet.edits = Self::items(&body);
				continue;
			}
			let cell = CellRef::parse(heading).map_err(|err| {
				bevyhow!("`## {heading}` names no cell: {err}")
			})?;
			if sheet.answers.iter().any(|answer| answer.cell == cell) {
				bevybail!("`## {heading}` is answered twice");
			}
			sheet.answers.push(CellAnswer {
				cell,
				answer: body.trim().to_string(),
			});
		}
		sheet.xok()
	}

	/// What is wrong with the sheet against the cells of its form: a cell
	/// the form has not, or one left without an answer.
	pub fn problems(&self, cells: &[FormCell]) -> Vec<String> {
		self.answers
			.iter()
			.filter_map(|answer| {
				match (
					cells.iter().any(|cell| cell.cell == answer.cell),
					answer.answer.is_empty(),
				) {
					(false, _) => Some(format!(
						"`{}` is no cell of the form a person may type into",
						answer.cell
					)),
					(true, true) => {
						Some(format!("`{}` holds no answer", answer.cell))
					}
					(true, false) => None,
				}
			})
			.collect()
	}

	/// Each `## ` heading of `text` with the lines under it, a heading inside
	/// a code fence being none.
	fn sections(text: &str) -> Vec<(&str, String)> {
		let mut sections = Vec::<(&str, String)>::new();
		let mut fenced = false;
		for line in text.lines() {
			if line.trim_start().starts_with("```") {
				fenced = !fenced;
			}
			match line.strip_prefix("## ").filter(|_| !fenced) {
				Some(heading) => sections.push((heading.trim(), String::new())),
				None => {
					if let Some((_, body)) = sections.last_mut() {
						body.push_str(line);
						body.push('\n');
					}
				}
			}
		}
		sections
	}

	/// The items of a list, an item's following lines joined onto it.
	fn items(body: &str) -> Vec<String> {
		let mut items = Vec::<String>::new();
		for line in body.lines().map(str::trim).filter(|line| !line.is_empty())
		{
			match (line.strip_prefix("- "), items.last_mut()) {
				(Some(item), _) => items.push(item.to_string()),
				(None, Some(item)) => {
					item.push(' ');
					item.push_str(line);
				}
				(None, None) => {}
			}
		}
		items
	}
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	#[beet::test]
	fn reads_answers_by_cell() {
		let sheet = AnswerSheet::parse(
			"# Answers\n\n## t0/1,0\n\nAcme Stalls\n\n## Start Here/3,2\n\n```md\n## not a cell\n```\n\n## Edits\n\n- Tick the first box\n  in 2.1.\n- Remove the red sentence.\n",
		)
		.unwrap();
		sheet
			.answers
			.iter()
			.map(|answer| (answer.cell.to_string(), answer.answer.clone()))
			.collect::<Vec<_>>()
			.xpect_eq(vec![
				("t0/1,0".to_string(), "Acme Stalls".to_string()),
				("Start Here/3,2".into(), "```md\n## not a cell\n```".into()),
			]);
		sheet.edits.xpect_eq(vec![
			"Tick the first box in 2.1.".to_string(),
			"Remove the red sentence.".into(),
		]);
		AnswerSheet::parse("## Business name\n\nAcme\n")
			.unwrap_err()
			.to_string()
			.xpect_contains("`## Business name` names no cell");
		AnswerSheet::parse("## t0/1,0\n\nA\n\n## t0/1,0\n\nB\n")
			.unwrap_err()
			.to_string()
			.xpect_contains("answered twice");
	}

	#[beet::test]
	fn names_what_the_form_lacks() {
		let cells = Fixture::cells_of(&Fixture::blank_form());
		AnswerSheet::parse(
			"## t0/1,0\n\nAcme\n\n## t0/0,0\n\n## t7/0,0\n\nX\n",
		)
		.unwrap()
		.problems(&cells)
		.xpect_eq(vec![
			"`t0/0,0` holds no answer".to_string(),
			"`t7/0,0` is no cell of the form a person may type into".into(),
		]);
	}
}
