use crate::prelude::*;
use crate::text_type::text_type;
use beet::prelude::*;

/// One statement about the subject, true of a strong subject and false of a
/// missing one, defined exactly once and never renamed: a row of a package's
/// `evals` table, keyed on its [`EvalId`]. The [`eval`](crate::eval) module
/// docs are its law.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct Eval {
	/// `namespace.slug`, global across a workspace and permanent.
	pub id: EvalId,
	/// Exactly one sentence, the thing judged or checked.
	pub statement: String,
	/// How the statement is decided: the scale as written, met or not, or four
	/// pinned lines.
	pub levels: Levels,
	/// Where the statement comes from, most authoritative first, at least one.
	pub sources: Vec<SourceTag>,
	/// Where in the subject the statement is expected to be satisfied; absent,
	/// the document named by the id's namespace.
	pub anchor: Option<Address>,
	/// One paragraph for a contradiction between sources, an accepted
	/// not-applicable answer or a warning to the grader, never a second
	/// statement.
	pub note: Option<String>,
}

impl Eval {
	/// Whether a check route decides this eval rather than a grader.
	pub fn is_checked(&self) -> bool { self.levels.check().is_some() }

	/// The address a grader reads: the anchor, else the document the id's
	/// namespace names.
	pub fn anchor_or_namespace(&self) -> Address {
		self.anchor
			.clone()
			.unwrap_or_else(|| Address::document(self.id.namespace()))
	}
}

/// A row of a package's `evals` table, keyed on its id.
impl TableStoreRow for Eval {
	fn table_name() -> SmolStr { "evals".into() }
	fn key(&self) -> TableKey { self.id.as_str().into() }
}

text_type!(
	/// An eval's id, `namespace.slug`, ie `market.competitors-named`: global
	/// across a workspace, so a rubric in one package cites an eval in another
	/// by id alone. The namespace groups evals, by the document they anchor in
	/// for a document package; the slug names one.
	EvalId,
	|text| match text.split_once('.') {
		Some((namespace, slug))
			if EvalId::is_slug(namespace) && EvalId::is_slug(slug) =>
		{
			OK
		}
		_ => bevybail!(
			"`{text}` is not an eval id, expected `namespace.slug` in lowercase \
			 letters, digits and single hyphens"
		),
	}
);

impl EvalId {
	/// Joins a namespace and a slug.
	pub fn new(namespace: &str, slug: &str) -> Result<Self> {
		Self::parse(format!("{namespace}.{slug}"))
	}

	/// The part before the dot, ie `market`.
	pub fn namespace(&self) -> &str {
		self.0
			.split_once('.')
			.map_or("", |(namespace, _)| namespace)
	}

	/// The part after the dot, ie `competitors-named`.
	pub fn slug(&self) -> &str {
		self.0.split_once('.').map_or("", |(_, slug)| slug)
	}

	/// Whether `text` is a slug: lowercase letters and digits in runs joined
	/// by single hyphens, ie `competitors-named`. A namespace, an eval's slug
	/// and a package's rubric id are slugs.
	pub fn is_slug(text: &str) -> bool {
		!text.is_empty()
			&& text.split('-').all(|run| {
				!run.is_empty()
					&& run.chars().all(|char| {
						char.is_ascii_lowercase() || char.is_ascii_digit()
					})
			})
	}
}

text_type!(
	/// Where a statement comes from, the text a citation carries in square
	/// brackets without them, ie `ref:sba` for `[ref:sba]` or `A1.3 transcript
	/// 00:05:26`. A tag's meaning is looked up in the source table of the
	/// package that owns the source, [`PackageManifest::sources`].
	SourceTag,
	|text| match !text.is_empty()
		&& text.trim() == text.as_str()
		&& !text.contains(['[', ']'])
	{
		true => OK,
		false => bevybail!(
			"`{text}` is not a source tag: it must be non-empty, with no \
			 surrounding whitespace and no square brackets"
		),
	}
);

text_type!(
	/// A place in the subject, `document` or `document#section`, ie
	/// `legal#risk-register`: a section's slug is its heading lowercased,
	/// punctuation dropped and spaces turned to hyphens
	/// ([`Address::slug`]). An eval's anchor, a data block's home, a claim's
	/// block and a grade's evidence are all addresses, and promoting a document
	/// to a directory changes none of them.
	Address,
	|text| {
		let (document, section) = match text.split_once('#') {
			Some((document, section)) => (document, Some(section)),
			None => (text.as_str(), None),
		};
		let part =
			|part: &str| !part.is_empty() && part.chars().all(is_address_char);
		match document.split('/').all(part) && section.is_none_or(part) {
			true => OK,
			false => bevybail!(
				"`{text}` is not an address, expected `document` or \
				 `document#section` in lowercase letters, digits and hyphens"
			),
		}
	}
);

impl Address {
	/// The address of a whole document.
	pub fn document(name: &str) -> Self { Self(name.into()) }

	/// The document part, ie `legal` for `legal#risk-register`.
	pub fn document_name(&self) -> &str {
		self.0
			.split_once('#')
			.map_or(&self.0, |(document, _)| document)
	}

	/// The section slug, ie `risk-register` for `legal#risk-register`.
	pub fn section(&self) -> Option<&str> {
		self.0.split_once('#').map(|(_, section)| section)
	}

	/// The addressing rule: a heading lowercased, everything but letters,
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

fn is_address_char(char: char) -> bool {
	char.is_ascii_lowercase() || char.is_ascii_digit() || char == '-'
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	#[beet::test]
	fn parses_ids() {
		let id = EvalId::parse("market.competitors-named").unwrap();
		id.namespace().xpect_eq("market");
		id.slug().xpect_eq("competitors-named");
		for bad in ["market", "Market.named", "market.-named", "a.b.c", ""] {
			EvalId::parse(bad).xpect_err();
		}
	}

	#[beet::test]
	fn parses_addresses() {
		let address = Address::parse("legal#risk-register").unwrap();
		address.document_name().xpect_eq("legal");
		address.section().xpect_eq(Some("risk-register"));
		Address::parse("market/customers")
			.unwrap()
			.section()
			.xpect_none();
		for bad in ["legal#", "#risk", "Legal", "legal#risk register", ""] {
			Address::parse(bad).xpect_err();
		}
	}

	#[beet::test]
	fn slugs_headings() {
		Address::slug("Owner's finances").xpect_eq("owners-finances");
		Address::slug("SWOT").xpect_eq("swot");
		Address::slug("Name and tagline").xpect_eq("name-and-tagline");
		Address::slug("Licences and codes").xpect_eq("licences-and-codes");
		Address::slug(" Cash  flow - forecast ").xpect_eq("cash-flow-forecast");
	}

	#[beet::test]
	fn parses_tags() {
		SourceTag::parse("A1.3 transcript 00:05:26").unwrap();
		for bad in ["", " demo", "[demo]"] {
			SourceTag::parse(bad).xpect_err();
		}
	}
}
