use crate::prelude::*;
use beet::prelude::*;

/// The params of [`SectionsCheck`].
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct SectionsCheckParams {
	/// The document's name.
	pub document: SmolStr,
	/// The headings, as written.
	pub headings: Vec<String>,
}

impl SectionsCheckParams {
	/// The verdict on `documents`.
	pub fn decide(&self, documents: &DocumentSet) -> CheckVerdict {
		let Some(document) = documents.get(&self.document) else {
			return CheckVerdict::fail(
				format!("{} missing", documents.path_name(&self.document)),
				[self.document.as_str()],
			);
		};
		let got = document
			.sections
			.iter()
			.map(|section| section.heading.as_str())
			.collect::<Vec<_>>();
		let want = self.headings.iter().map(String::as_str).collect::<Vec<_>>();
		if got == want {
			return CheckVerdict::pass(format!(
				"{} sections in order",
				want.len()
			));
		}
		let missing = want
			.iter()
			.filter(|heading| !got.contains(heading))
			.copied();
		let extra = got
			.iter()
			.filter(|heading| !want.contains(heading))
			.copied();
		let mut parts = Vec::new();
		for (label, headings) in [
			("missing", missing.collect::<Vec<_>>()),
			("extra", extra.collect()),
		] {
			if !headings.is_empty() {
				parts.push(format!("{label}: {}", headings.join(", ")));
			}
		}
		if parts.is_empty() {
			parts.push("out of order".into());
		}
		CheckVerdict::fail(parts.join("; "), [self.document.as_str()])
	}
}

/// `check/sections`: passes when the document's `##` headings are exactly
/// these, in this order, and names the missing, the extra or the misordered.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
#[require(
	PathPartial = PathPartial::new("sections"),
	ParamsPartial = ParamsPartial::new::<SectionsCheckParams>()
)]
pub async fn SectionsCheck(cx: ActionContext<Request>) -> Result<Response> {
	DocumentSet::answer_check(&cx, SectionsCheckParams::decide).await
}
