use crate::prelude::*;
use beet::prelude::*;

/// The params of [`TaglineCheck`].
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct TaglineCheckParams {
	/// The document's name.
	pub document: SmolStr,
}

impl TaglineCheckParams {
	/// The verdict on `documents`.
	pub fn decide(&self, documents: &DocumentSet) -> CheckVerdict {
		let fail = |detail: String| {
			CheckVerdict::fail(detail, [self.document.as_str()])
		};
		let Some(document) = documents.get(&self.document) else {
			return fail(format!(
				"{} missing",
				documents.path_name(&self.document)
			));
		};
		match (&document.title, &document.tagline) {
			(None, _) => fail("no level one heading".into()),
			(Some(_), Some(tagline)) => CheckVerdict::pass(tagline),
			(Some(_), None) => {
				fail("no emphasised line stands beneath the title".into())
			}
		}
	}
}

/// `check/tagline`: passes when one emphasised line, `*like this*`, stands
/// alone beneath the title.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
#[require(
	PathPartial = PathPartial::new("tagline"),
	ParamsPartial = ParamsPartial::new::<TaglineCheckParams>()
)]
pub async fn TaglineCheck(cx: ActionContext<Request>) -> Result<Response> {
	DocumentSet::answer_check(&cx, TaglineCheckParams::decide).await
}
