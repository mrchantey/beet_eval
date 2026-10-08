use crate::prelude::*;
use beet::prelude::*;

/// The params of [`AsksCheck`].
#[derive(Debug, Default, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct AsksCheckParams {}

impl AsksCheckParams {
	/// The most positions a failure lists.
	const SHOWN: usize = 6;

	/// The verdict on `documents`. An open ask is no shape failure, so the
	/// verdict lays it at no document; the triage counts asks on its own.
	pub fn decide(&self, documents: &mut DocumentSet) -> CheckVerdict {
		let asks = documents.asks();
		if asks.is_empty() {
			return CheckVerdict::pass("no open asks");
		}
		let mut positions = asks
			.iter()
			.map(|(document, ask)| {
				format!("{}:{}", documents.path_of(document), ask.line)
			})
			.collect::<Vec<_>>();
		positions.dedup();
		let more = match positions.len() > Self::SHOWN {
			true => ", ...",
			false => "",
		};
		positions.truncate(Self::SHOWN);
		CheckVerdict::fail(
			format!("{} open: {}{more}", asks.len(), positions.join(", ")),
			Vec::<SmolStr>::new(),
		)
	}
}

/// `check/asks`: passes when no ask is open anywhere under the documents
/// directory, and lists where the open ones are.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
#[require(
	PathPartial = PathPartial::new("asks"),
	ParamsPartial = ParamsPartial::new::<AsksCheckParams>()
)]
pub async fn AsksCheck(cx: ActionContext<Request>) -> Result<Response> {
	DocumentSet::answer_check(&cx, AsksCheckParams::decide).await
}
