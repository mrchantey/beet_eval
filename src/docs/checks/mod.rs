//! The document check kinds, each a route under `check/`: its action here
//! beside its params type, and `<CheckRoutes/>` mounting them all. A route
//! reads its params, reads the documents through the workspace store above
//! it, and answers a [`CheckVerdict`] as JSON;
//! `eval/results` calls the route for every checked eval through the route
//! tree, and a person may call one directly, ie `check/sections
//! --document=brand --headings=Voice`. A document is named without its
//! extension, `brand` for `brand.md` or `brand/index.md` under the documents
//! directory, so promotion never breaks a check.
//!
//! | Route | Params | Passes when |
//! |---|---|---|
//! | [`DocumentCheck`] | [`DocumentCheckParams`] | the document exists as a file or a directory with an index |
//! | [`FrontmatterCheck`] | [`FrontmatterCheckParams`] | every document resolves and its frontmatter carries every key with a value |
//! | [`H1Check`] | [`H1CheckParams`] | the first block after the frontmatter is a level one heading |
//! | [`TaglineCheck`] | [`TaglineCheckParams`] | one emphasised line stands alone beneath the title |
//! | [`SummaryCheck`] | [`SummaryCheckParams`] | every document has a paragraph between its title, or tagline, and its first `##` |
//! | [`AgreesCheck`] | [`AgreesCheckParams`] | the document's title or tagline appears verbatim in the section that owns it, or both are still open asks |
//! | [`AsksCheck`] | [`AsksCheckParams`] | no ask is open anywhere under the documents directory |
//! | [`SectionsCheck`] | [`SectionsCheckParams`] | the document's `##` headings are exactly these, in order |
//! | [`BlockCheck`] | [`BlockCheckParams`] | the named `csv` block is defined once, under its section, with exactly these columns, each typed cell of its type |
//!
//! A verdict lays a failure at the documents whose shape it concerns, which
//! the triage counts as shape failures; an open ask is laid at none, since
//! the triage counts asks on their own.
//!
//! A document package's
//! [`Outline`] generates the
//! `document`, `sections` and `block` evals of its documents and blocks and
//! the `frontmatter` and `summary` evals over the set; the package writes the
//! rest as rows. `eval/check` validates every eval's params against its
//! route's `ParamsPartial`, and refuses an anchor, a block's home or a check's
//! section naming a document or a section the outline does not declare, so
//! the outline's sections stay the one list of addresses.
mod agrees;
mod asks;
mod block;
mod document;
mod frontmatter;
mod h1;
mod sections;
mod summary;
mod tagline;
pub use agrees::*;
pub use asks::*;
pub use block::*;
pub use document::*;
pub use frontmatter::*;
pub use h1::*;
pub use sections::*;
pub use summary::*;
pub use tagline::*;

use crate::prelude::*;
use beet::exports::bevy::reflect::Typed;
use beet::prelude::*;

/// `<CheckRoutes/>`: every document check kind as a child route, authored
/// under the `check` route the engine calls them by.
///
/// ```bsx
/// <Route path="check"><CheckRoutes/></Route>
/// ```
#[template]
pub fn CheckRoutes() -> impl Bundle {
	children![
		DocumentCheck,
		FrontmatterCheck,
		H1Check,
		TaglineCheck,
		SummaryCheck,
		AgreesCheck,
		AsksCheck,
		SectionsCheck,
		BlockCheck,
	]
}

impl DocumentSet {
	/// Answers a check route's request: its params read, the workspace's
	/// documents read through the store above the route, and the verdict
	/// `decide` reaches answered as JSON.
	pub async fn answer_check<P: FromReflect + Typed>(
		cx: &ActionContext<Request>,
		decide: fn(&P, &mut DocumentSet) -> CheckVerdict,
	) -> Result<Response> {
		let params = cx.input.parse_params::<P>()?;
		let store = LoadedWorkspace::store_of(&cx.caller).await?;
		let manifest = Workspace::read(&store).await?;
		let mut documents = DocumentSet::load(
			&store.with_subdir(manifest.docs.clone()),
			manifest.docs,
		)
		.await?;
		Response::ok_json(&decide(&params, &mut documents))
	}
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	/// Every check's verdict in a run of the fixture.
	async fn checks(fixture: &mut Fixture) -> Vec<CheckOutcome> {
		MediaType::Json
			.deserialize::<Results>(
				fixture
					.ok("eval/results --accept=application/json")
					.await
					.as_bytes(),
			)
			.unwrap()
			.checks
	}

	/// The brand document written in Word rather than markdown passes and
	/// fails every check alike: its title and sections by their heading
	/// styles, its tagline's agreement by its emphasis, its open ask by its
	/// words. Only its frontmatter differs, a Word file's being its core
	/// properties, which this one leaves empty.
	#[beet::test]
	async fn checks_a_word_document_like_markdown() {
		let mut fixture = Fixture::new().await;
		let markdown = checks(&mut fixture).await;
		fixture
			.store
			.remove(&RelPath::new("docs/brand.md"))
			.await
			.unwrap();
		fixture
			.store
			.insert(
				&RelPath::new("docs/brand.docx"),
				OoxmlFile::word(
					"<w:p><w:pPr><w:pStyle w:val=\"Title\"/></w:pPr><w:r><w:t>Brand</w:t></w:r></w:p>\
					 <w:p><w:r><w:t>The name is Acme Stalls, and the tagline promises a stall that arrives ready to trade.</w:t></w:r></w:p>\
					 <w:p><w:pPr><w:pStyle w:val=\"Heading2\"/></w:pPr><w:r><w:t>Name and tagline</w:t></w:r></w:p>\
					 <w:p><w:r><w:t xml:space=\"preserve\">Acme Stalls, </w:t></w:r>\
					 <w:r><w:rPr><w:i/></w:rPr><w:t>Stalls that come fitted</w:t></w:r><w:r><w:t>.</w:t></w:r></w:p>\
					 <w:p><w:pPr><w:pStyle w:val=\"Heading2\"/></w:pPr><w:r><w:t>Voice</w:t></w:r></w:p>\
					 <w:p><w:r><w:t>TODO(ask): how plain or playful the voice is.</w:t></w:r></w:p>",
				)
				.unwrap()
				.bytes()
				.to_vec(),
			)
			.await
			.unwrap();
		let word = checks(&mut fixture).await;
		for (markdown, word) in markdown.iter().zip(&word) {
			match markdown.eval.slug() {
				"frontmatter-complete" => {
					word.detail.clone().xpect_contains(
						"docs/brand: created, updated, authors",
					);
				}
				_ => {
					(markdown.eval.clone(), markdown.pass)
						.xpect_eq((word.eval.clone(), word.pass));
				}
			}
		}
		word.iter()
			.find(|check| check.eval.slug() == "no-open-asks")
			.unwrap()
			.detail
			.clone()
			.xpect_contains("docs/brand.docx:6");
	}
}
