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
	pub fn decide(&self, documents: &mut DocumentSet) -> CheckVerdict {
		let fail = |detail: String| {
			CheckVerdict::fail(detail, [self.document.as_str()])
		};
		let Some(root) =
			documents.get(&self.document).map(|document| document.root)
		else {
			return fail(format!(
				"{} missing",
				documents.path_name(&self.document)
			));
		};
		match documents.query(|query| (query.title(root), query.tagline(root)))
		{
			(None, _) => fail("no level one heading".into()),
			(Some(_), Some(tagline)) => CheckVerdict::pass(tagline.as_str()),
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
