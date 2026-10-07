use crate::prelude::*;
use beet::prelude::*;

/// The params of [`H1Check`].
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct H1CheckParams {
	/// The document's name.
	pub document: SmolStr,
}

impl H1CheckParams {
	/// The verdict on `documents`.
	pub fn decide(&self, documents: &DocumentSet) -> CheckVerdict {
		let fail = |detail: String| {
			CheckVerdict::fail(detail, [self.document.as_str()])
		};
		match documents.get(&self.document) {
			None => fail(format!("{} missing", documents.path_name(&self.document))),
			Some(document) => match &document.title {
				Some(title) => CheckVerdict::pass(title.as_str()),
				None => fail(
					"the first line after the frontmatter is not a level one heading"
						.into(),
				),
			},
		}
	}
}

/// `check/h1`: passes when the first block after the frontmatter is a level
/// one heading.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
#[require(
	PathPartial = PathPartial::new("h1"),
	ParamsPartial = ParamsPartial::new::<H1CheckParams>()
)]
pub async fn H1Check(cx: ActionContext<Request>) -> Result<Response> {
	DocumentSet::answer_check(&cx, H1CheckParams::decide).await
}
