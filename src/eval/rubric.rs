use crate::prelude::*;
use crate::text_type::text_type;
use beet::prelude::*;

/// A profile over evals for one reader: the level each cited eval must reach,
/// and the structural lines that reader's form demands, under the form's own
/// headings. A row of a package's `rubrics` table, keyed on its id.
///
/// A rubric's citations are read off the one [`Results`] grading `docs/`
/// produces, never off a rendered form; its structural lines are checked on the
/// rendered form, never on `docs/`.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct Rubric {
	/// Unique within its package, ie `01-business-plan`; the workspace names it
	/// `<package>/<id>`, a [`RubricRef`].
	pub id: SmolStr,
	/// Who reads the form and on what basis, ie `an assessor marking the form
	/// satisfactory`.
	pub reader: String,
	/// What the form is, where it goes, what it feeds and what feeds it.
	pub intro: String,
	/// Every source the rubric was built from, most authoritative first.
	pub sources: Vec<String>,
	/// The form's headings, in the form's order.
	pub sections: Vec<RubricSection>,
}

impl Rubric {
	/// Every citation in the rubric, in heading order.
	pub fn citations(&self) -> impl Iterator<Item = &Citation> {
		self.sections.iter().flat_map(RubricSection::citations)
	}
}

/// A row of a package's `rubrics` table, keyed on its id.
impl TableStoreRow for Rubric {
	fn table_name() -> SmolStr { "rubrics".into() }
	fn key(&self) -> TableKey { self.id.as_str().into() }
}

/// One heading of a form, carrying the form's own label exactly so a rubric
/// address is an address on the form, with what the reader looks for under it
/// and the headings nested beneath it.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct RubricSection {
	/// The address prefix of the heading's structural lines, unique in the
	/// rubric: the form's own number where it has one (`2.3`, `4.2 A`, `14.d`),
	/// else a word for it (`whole` for the whole document).
	pub label: SmolStr,
	/// The heading as the form prints it, ie `2.3 Competitor Analysis`.
	pub title: String,
	/// What the section asks in the form's terms and what the reader looks for.
	pub prose: String,
	/// The evals the reader holds this section to, each at a level.
	pub citations: Vec<Citation>,
	/// Checkable sentences about the rendered form. Append only: a line's
	/// address is `label.n` by its position, which results cite.
	pub structural: Vec<StructuralLine>,
	/// What gets the form sent back, each with its source tags inline.
	pub sent_back_when: Vec<String>,
	/// The other sections or forms that must carry the same fact.
	pub agrees_with: Vec<String>,
	/// What the demonstration or a good answer does here.
	pub example: Option<String>,
	/// What a stronger answer than the reader's minimum would add.
	pub beyond_minimum: Option<String>,
	/// The headings nested under this one, in the form's order.
	pub sections: Vec<RubricSection>,
}

impl RubricSection {
	/// This section's citations and its descendants', in heading order.
	pub fn citations(&self) -> Box<dyn Iterator<Item = &Citation> + '_> {
		Box::new(
			self.citations
				.iter()
				.chain(self.sections.iter().flat_map(Self::citations)),
		)
	}
}

/// An eval cited at the level this reader demands, 1 to 3, and 2 for a binary
/// eval. The rubric's prose says why; the eval says what.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct Citation {
	/// The eval, by its workspace-global id.
	pub eval: EvalId,
	/// The level it must reach.
	pub level: EvalLevel,
}

/// One checkable sentence about the rendered form: a cell filled, a count of
/// rows, a sentence deleted, a mark made bold.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct StructuralLine {
	/// The requirement, one sentence.
	pub text: String,
	/// Where it comes from, at least one tag, most authoritative first.
	pub sources: Vec<SourceTag>,
}

impl StructuralLine {
	/// The line as a rubric writes it, its source tags after the text, ie
	/// `Every red sentence is deleted. [template red]`.
	pub fn written(&self) -> String {
		std::iter::once(self.text.clone())
			.chain(self.sources.iter().map(|source| format!("[{source}]")))
			.collect::<Vec<_>>()
			.join(" ")
	}
}

text_type!(
	/// A rubric named across a workspace, `<package>/<rubric>`, ie
	/// `course/01-business-plan`: what `eval/project` and `eval/build`
	/// take and [`Results`] reports by.
	RubricRef,
	|text| match text.split_once('/') {
		Some((package, rubric))
			if PackageManifest::is_name(package) && EvalId::is_slug(rubric) =>
		{
			OK
		}
		_ => bevybail!(
			"`{text}` is not a rubric reference, expected `<package>/<rubric>`"
		),
	}
);

impl RubricRef {
	/// Names `rubric` in `package`.
	pub fn new(package: &str, rubric: &str) -> Result<Self> {
		Self::parse(format!("{package}/{rubric}"))
	}

	/// The package part, ie `course`.
	pub fn package(&self) -> &str {
		self.0.split_once('/').map_or("", |(package, _)| package)
	}

	/// The rubric's id within its package, ie `01-business-plan`.
	pub fn rubric(&self) -> &str {
		self.0.split_once('/').map_or("", |(_, rubric)| rubric)
	}
}
