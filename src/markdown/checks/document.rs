use crate::prelude::*;
use beet::prelude::*;

/// The params of [`DocumentCheck`].
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct DocumentCheckParams {
	/// The document's name, ie `brand`.
	pub document: SmolStr,
}

impl DocumentCheckParams {
	/// The verdict on `documents`.
	pub fn decide(&self, documents: &DocumentSet) -> CheckVerdict {
		match documents.get(&self.document) {
			Some(document) => CheckVerdict::pass(documents.path_of(document)),
			None => {
				let path = documents.path_name(&self.document);
				CheckVerdict::fail(
					format!("{path}.md or {path}/index.md missing"),
					[self.document.as_str()],
				)
			}
		}
	}
}

/// `check/document`: passes when the document exists, as `<document>.md` or
/// `<document>/index.md` under the documents directory, so promotion never
/// breaks it.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
#[require(
	PathPartial = PathPartial::new("document"),
	ParamsPartial = ParamsPartial::new::<DocumentCheckParams>()
)]
pub async fn DocumentCheck(cx: ActionContext<Request>) -> Result<Response> {
	DocumentSet::answer_check(&cx, DocumentCheckParams::decide).await
}
