use crate::prelude::*;
use beet::prelude::*;

/// The params of [`SummaryCheck`].
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct SummaryCheckParams {
	/// The documents' names.
	pub documents: Vec<SmolStr>,
}

impl SummaryCheckParams {
	/// The verdict on `documents`.
	pub fn decide(&self, documents: &DocumentSet) -> CheckVerdict {
		let mut problems = Vec::new();
		let mut failing = Vec::new();
		for name in &self.documents {
			let path = documents.path_name(name);
			let problem = match documents.get(name) {
				None => Some(format!("{path}: missing")),
				Some(document) if document.title.is_none() => {
					Some(format!("{path}: no title"))
				}
				Some(document) if document.summary.is_none() => Some(format!(
					"{path}: no paragraph between the title and the first section"
				)),
				Some(_) => None,
			};
			if let Some(problem) = problem {
				problems.push(problem);
				failing.push(name.clone());
			}
		}
		match problems.is_empty() {
			true => CheckVerdict::pass(format!(
				"{} documents open with a summary",
				self.documents.len()
			)),
			false => CheckVerdict::fail(problems.join("; "), failing),
		}
	}
}

/// `check/summary`: passes when every document has a paragraph between its
/// title, or tagline, and its first `##`.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
#[require(
	PathPartial = PathPartial::new("summary"),
	ParamsPartial = ParamsPartial::new::<SummaryCheckParams>()
)]
pub async fn SummaryCheck(cx: ActionContext<Request>) -> Result<Response> {
	DocumentSet::answer_check(&cx, SummaryCheckParams::decide).await
}
