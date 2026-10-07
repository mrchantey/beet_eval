use beet::prelude::*;

/// A `##` section of a
/// [`MarkdownDocument`](crate::prelude::MarkdownDocument): an anchor that does
/// not move, present even when its body says why it does not apply.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct Section {
	/// The heading as written, ie `Owner's finances`.
	pub heading: String,
	/// The heading's slug, the section half of an
	/// [`Address`](crate::prelude::Address), ie `owners-finances`.
	pub slug: SmolStr,
	/// The heading's line in its file, from 1.
	pub line: u32,
	/// The markdown below the heading, up to the next `##`.
	pub body: String,
}

impl Section {
	/// The addressing rule: the heading lowercased, everything but letters,
	/// digits, spaces and hyphens dropped, runs of spaces turned to one hyphen
	/// and runs of hyphens collapsed, ie `Owner's finances` to
	/// `owners-finances`.
	pub fn slug(heading: &str) -> SmolStr {
		heading
			.to_lowercase()
			.chars()
			.filter(|char| {
				char.is_ascii_lowercase()
					|| char.is_ascii_digit()
					|| *char == ' ' || *char == '-'
			})
			.collect::<String>()
			.split_whitespace()
			.collect::<Vec<_>>()
			.join("-")
			.split('-')
			.filter(|run| !run.is_empty())
			.collect::<Vec<_>>()
			.join("-")
			.into()
	}
}

/// An open question to the owner, written inline where its answer belongs as
/// `TODO(ask fact: ...)` or `TODO(ask decision: ...)`, a bare `TODO(ask: ...)`
/// being a fact. A section with one open is graded at most 1 on every eval
/// anchored there.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct Ask {
	/// Who can answer it.
	pub kind: AskKind,
	/// The question, as written after the colon.
	pub text: String,
	/// Its line in its file, from 1.
	pub line: u32,
	/// The slug of the section it sits in, absent above the first.
	pub section: Option<SmolStr>,
}

impl Ask {
	/// The marker every ask opens with.
	const MARKER: &str = "TODO(ask";

	/// Every ask in one line, in order, with its kind and question. Two
	/// spellings are read: the question inside the parentheses, `TODO(ask
	/// decision: which market first?)`, and the question after them running to
	/// the end of the line or the next ask, `TODO(ask): the tagline`, which is
	/// what a scaffold writes. A kind of `fact` or `decision` follows `ask`;
	/// none is a fact.
	pub fn find_all(line: &str) -> Vec<(AskKind, String)> {
		let mut asks = Vec::new();
		let mut rest = line;
		while let Some(start) = rest.find(Self::MARKER) {
			let after = &rest[start + Self::MARKER.len()..];
			let (kind, after) = match after.trim_start() {
				spelled if spelled.starts_with("fact") => {
					(AskKind::Fact, &spelled[4..])
				}
				spelled if spelled.starts_with("decision") => {
					(AskKind::Decision, &spelled[8..])
				}
				_ => (AskKind::Fact, after),
			};
			let (question, remainder) = match after.strip_prefix(':') {
				// inside the parentheses, to the one that closes them
				Some(inside) => {
					let mut depth = 0;
					let close = inside.char_indices().find(|(_, char)| {
						match char {
							'(' => depth += 1,
							')' if depth == 0 => return true,
							')' => depth -= 1,
							_ => {}
						}
						false
					});
					match close {
						Some((at, _)) => (&inside[..at], &inside[at + 1..]),
						None => (inside, ""),
					}
				}
				// after the parentheses, to the next ask or the line's end
				None => {
					let after = after.strip_prefix(')').unwrap_or(after);
					let after = after.strip_prefix(':').unwrap_or(after);
					let end = after.find(Self::MARKER).unwrap_or(after.len());
					(&after[..end], &after[end..])
				}
			};
			asks.push((kind, question.trim().to_string()));
			rest = remainder;
		}
		asks
	}
}

/// Who can answer an [`Ask`].
#[derive(
	Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize,
)]
pub enum AskKind {
	/// A source could answer it, so the writer offers a default from one.
	Fact,
	/// Only the owner can, so it carries no default and the coach challenges it
	/// instead.
	Decision,
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	#[beet::test]
	fn finds_asks() {
		Ask::find_all("Name: TODO(ask): the name. More text")
			.xpect_eq(vec![(AskKind::Fact, "the name. More text".to_string())]);
		Ask::find_all(
			"TODO(ask decision: which (first) market?) then TODO(ask: why)",
		)
		.xpect_eq(vec![
			(AskKind::Decision, "which (first) market?".to_string()),
			(AskKind::Fact, "why".into()),
		]);
		Ask::find_all("TODO(ask): one. TODO(ask): two.")
			.len()
			.xpect_eq(2);
		Ask::find_all("no asks").xpect_empty();
	}

	#[beet::test]
	fn slugs_headings() {
		Section::slug("Owner's finances").xpect_eq("owners-finances");
		Section::slug("SWOT").xpect_eq("swot");
		Section::slug("Name and tagline").xpect_eq("name-and-tagline");
		Section::slug("Licences and codes").xpect_eq("licences-and-codes");
		Section::slug(" Cash  flow - forecast ").xpect_eq("cash-flow-forecast");
	}
}
