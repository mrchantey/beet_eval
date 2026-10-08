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
	pub fn decide(&self, documents: &mut DocumentSet) -> CheckVerdict {
		let mut problems = Vec::new();
		let mut failing = Vec::new();
		for name in &self.documents {
			let path = documents.path_name(name);
			let head =
				documents
					.get(name)
					.map(|document| document.root)
					.map(|root| {
						documents.query(|query| {
							(
								query.title(root).is_some(),
								query.summary(root).is_some(),
							)
						})
					});
			let problem = match head {
				None => Some(format!("{path}: missing")),
				Some((false, _)) => Some(format!("{path}: no title")),
				Some((true, false)) => Some(format!(
					"{path}: no paragraph between the title and the first section"
				)),
				Some((true, true)) => None,
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
