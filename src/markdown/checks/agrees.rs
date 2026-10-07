use crate::prelude::*;
use beet::prelude::*;

/// The params of [`AgreesCheck`].
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct AgreesCheckParams {
	/// The document whose title or tagline is the source of truth, ie `index`.
	pub document: SmolStr,
	/// Which of the two must agree.
	pub part: TitlePart,
	/// The section that owns it in detail, ie `brand#name-and-tagline`.
	pub section: Address,
}

/// The two parts of a document's head that another section may own.
#[derive(
	Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize,
)]
pub enum TitlePart {
	/// The level one heading.
	Title,
	/// The emphasised line beneath it.
	Tagline,
}

impl TitlePart {
	/// The part as a sentence names it.
	pub fn word(&self) -> &'static str {
		match self {
			Self::Title => "title",
			Self::Tagline => "tagline",
		}
	}
}

impl AgreesCheckParams {
	/// The verdict on `documents`.
	pub fn decide(&self, documents: &DocumentSet) -> CheckVerdict {
		let target = self.section.document_name();
		let fail = |detail: String| {
			CheckVerdict::fail(detail, [self.document.as_str(), target])
		};
		let source = documents.path_name(&self.document);
		let Some(document) = documents.get(&self.document) else {
			return fail(format!("{source} missing"));
		};
		if document.title.is_none() {
			return fail(format!("{source} has no title"));
		}
		let text = match self.part {
			TitlePart::Title => document.title.clone(),
			TitlePart::Tagline => document.tagline.clone(),
		};
		let Some(text) = text else {
			return fail(format!("{source} has no {}", self.part.word()));
		};
		let Some(owner) = documents.get(target) else {
			return fail(format!("{target} missing"));
		};
		let slug = self.section.section().unwrap_or_default();
		let Some(section) = owner.section(slug) else {
			return fail(format!(
				"no section #{slug} in {}",
				documents.path_of(owner)
			));
		};
		let undecided = !Ask::find_all(&text).is_empty()
			&& !Ask::find_all(&section.body).is_empty();
		match section.body.replace(['*', '_', '`'], "").contains(&text) {
			true => CheckVerdict::pass(format!(
				"\"{text}\" appears under {}",
				self.section
			)),
			false if undecided => CheckVerdict::pass(format!(
				"undecided: \"{text}\" and {} both carry an open ask",
				self.section
			)),
			false => fail(format!("\"{text}\" is not under {}", self.section)),
		}
	}
}

/// `check/agrees`: passes when the document's title or tagline appears
/// verbatim, emphasis aside, in the section of the document that owns it, or
/// when the part is still an open ask and the section carries one too, so an
/// undecided value is reported by `check/asks` alone. The index is the source
/// of truth for both.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
#[require(
	PathPartial = PathPartial::new("agrees"),
	ParamsPartial = ParamsPartial::new::<AgreesCheckParams>()
)]
pub async fn AgreesCheck(cx: ActionContext<Request>) -> Result<Response> {
	DocumentSet::answer_check(&cx, AgreesCheckParams::decide).await
}
