use super::*;
use crate::prelude::*;
use beet::prelude::*;

/// `eval/deck <path>`: the [`DeckTriage`] of the slide deck at `path` in the
/// store, ie a suite's `assets/week-1.pptx`, read before transcribing it
/// beside `view <path>`'s dump: the [`DeckTriageReport`] for a markup
/// `Accept` and the triage for a serde one.
#[action(route = "deck/*file")]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
pub async fn EvalDeck(
	/// The deck's path in the store, the path after `deck`.
	#[param]
	file: RelPath,
	cx: ActionContext<Request>,
) -> Result<DataPage<DeckTriage>> {
	let deck = cx
		.caller
		.with_state::<AncestorQuery<&BlobStore>, _>(|entity, stores| {
			stores.get(entity).cloned()
		})
		.await??
		.blob(file.clone())
		.get_media()
		.await?;
	if deck.media_type() != &MediaType::Pptx {
		bevybail!("`{file}` is no slide deck");
	}
	let triage = parsed_with(&cx.caller, deck, |world, root| {
		DeckTriage::of(world, root).xok()
	})
	.await?;
	let report = rsx! {
		<DeckTriageReport file=file.to_string() triage=triage.clone()/>
	};
	DataPage::new(&cx.caller, report, triage).await
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	#[beet::test]
	async fn triages_a_deck_in_the_store() {
		let mut fixture = Fixture::new().await;
		let deck = Fixture::written(
			&MediaBytes::new_html(
				"<article data-card><h1>Week 1</h1><p>Welcome</p><hr><p>Say hello.</p></article>",
			),
			MediaType::Pptx,
		);
		fixture
			.store
			.insert(
				&RelPath::new("suites/acme/assets/week-1.pptx"),
				deck.bytes().to_vec(),
			)
			.await
			.unwrap();
		fixture
			.ok("eval/deck suites/acme/assets/week-1.pptx")
			.await
			.xpect_contains("# Triage of suites/acme/assets/week-1.pptx\n")
			.xpect_contains("1 slides, counted from 0")
			.xpect_contains("| 0 | Week 1 |  | 3 |  | 0% |  | 2 | picture |");
		fixture
			.refused("eval/deck docs/brand.md")
			.await
			.xpect_contains("is no slide deck");
	}
}
