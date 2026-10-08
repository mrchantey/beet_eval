use super::*;
use crate::prelude::*;
use beet::prelude::*;

/// Request params for [`EvalNew`], surfaced in `--help`.
#[derive(Reflect)]
struct NewParams {
	/// The subject's name, the index's title: the `:name` path segment.
	name: SmolStr,
	/// Who writes the documents, their frontmatter's author: the `:author`
	/// path segment.
	author: SmolStr,
}

/// `eval/new <name> <author>`: lays out the documents directory from the
/// outline, one file per document with its frontmatter, title and every
/// section, an empty data block under each section that owns one, a tagline
/// line where a check asks for one, and an ask in every body, so `check/asks`
/// fails until the clerk has been through. Refuses a directory that is not
/// empty.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
#[require(
	PathPartial = PathPartial::new("new/:name/:author"),
	ParamsPartial = ParamsPartial::new::<NewParams>()
)]
pub async fn EvalNew(cx: ActionContext<Request>) -> Result<Response> {
	let params = cx.input.parse_params::<NewParams>()?;
	let workspace = LoadedWorkspace::of(&cx.caller).await?;
	workspace.require_clean()?;
	let docs = workspace.docs();
	if docs.store_exists().await.unwrap_or(false)
		&& !docs.list().await?.is_empty()
	{
		return refusal(format!(
			"{}/ is not empty; refusing to scaffold over it\n",
			workspace.manifest.docs
		))
		.xok();
	}
	let outline = workspace.outline()?;
	let taglined = workspace
		.evals
		.iter()
		.filter_map(|packaged| packaged.eval.levels.check())
		.filter(|check| check.route.as_str() == "tagline")
		.filter_map(|check| {
			check.params.clone().into_serde::<TaglineCheckParams>().ok()
		})
		.map(|params| params.document)
		.collect::<Vec<_>>();
	let today = Date::today();
	let mut written = String::new();
	for spec in &outline.documents {
		let title = match spec.name.as_str() {
			"index" => params.name.to_string(),
			name => {
				let mut chars = name.chars();
				chars
					.next()
					.map(|first| first.to_uppercase().chain(chars).collect())
					.unwrap_or_default()
			}
		};
		let sections = spec
			.sections
			.iter()
			.map(|section| {
				let address = format!(
					"{}#{}",
					spec.name,
					Address::slug(&section.heading)
				);
				ScaffoldSection {
					heading: section.heading.clone(),
					blocks: outline
						.blocks
						.iter()
						.filter(|block| block.lives_in.as_str() == address)
						.map(|block| ScaffoldBlock {
							name: block.name.clone(),
							header: block
								.columns
								.iter()
								.map(|column| column.name.as_str())
								.collect::<Vec<_>>()
								.join(","),
						})
						.collect(),
				}
			})
			.collect::<Vec<_>>();
		let scaffold = rsx! {
			<ScaffoldDocument
				name=spec.name.to_string()
				title=title
				tagline={taglined.contains(&spec.name)}
				sections=sections
			/>
		};
		// the frontmatter is data, the body a render of the scaffold's scene
		let frontmatter = format!(
			"---\ncreated: {today}\nupdated: {today}\nauthors: [{}]\n---\n\n",
			params.author
		);
		let body = markdown_of(&cx.caller, scaffold).await?;
		let path = RelPath::new(format!("{}.md", spec.name));
		docs.insert(&path, format!("{frontmatter}{body}")).await?;
		written.push_str(&format!("{}/{path}\n", workspace.manifest.docs));
	}
	Response::ok_text(written).xok()
}

/// One section of a [`ScaffoldDocument`]: its heading and the empty data
/// blocks it owns.
#[derive(Debug, Default, Clone, PartialEq, Reflect)]
pub struct ScaffoldSection {
	/// The heading as the outline writes it.
	pub heading: SmolStr,
	/// The blocks the outline puts here.
	pub blocks: Vec<ScaffoldBlock>,
}

/// An empty data block of a scaffold: its name and its header row.
#[derive(Debug, Default, Clone, PartialEq, Reflect)]
pub struct ScaffoldBlock {
	/// The block's name, ie `price-list`.
	pub name: SmolStr,
	/// Its columns' names, comma separated.
	pub header: String,
}

/// A document as `eval/new` lays it out: its title, an ask for its tagline
/// where a check asks for one, an ask for its summary, and every section with
/// an ask and the empty data blocks it owns, so `check/asks` fails until the
/// clerk has been through.
#[template]
pub fn ScaffoldDocument(
	name: String,
	title: String,
	tagline: bool,
	sections: Vec<ScaffoldSection>,
) -> impl Bundle {
	let tagline =
		tagline.then(|| rsx! { <p><em>"TODO(ask): the tagline"</em></p> });
	let sections = sections
		.iter()
		.map(|section| {
			let blocks = section
				.blocks
				.iter()
				.map(|block| {
					rsx! {
						<pre>
							<code {Attribute::bundle("data-info", format!("csv {}", block.name))}>
								{block.header.clone()}
							</code>
						</pre>
					}
				})
				.collect::<Vec<_>>();
			rsx! {
				<h2>{section.heading.to_string()}</h2>
				<p>{format!("TODO(ask): {}.", section.heading)}</p>
				{blocks}
			}
		})
		.collect::<Vec<_>>();
	rsx! {
		<h1>{title}</h1>
		{tagline}
		<p>{format!("TODO(ask): the summary paragraph of {name}, its conclusions in a few sentences.")}</p>
		{sections}
	}
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	#[beet::test]
	async fn scaffolds_from_the_outline() {
		let mut fixture = Fixture::new().await;
		fixture
			.refused("eval/new Acme Ada")
			.await
			.xpect_contains("not empty");
		for path in fixture
			.store
			.with_subdir(RelPath::new("docs"))
			.list()
			.await
			.unwrap()
		{
			fixture
				.store
				.remove(&RelPath::new("docs").join(path.as_str()))
				.await
				.unwrap();
		}
		fixture
			.ok("eval/new Acme\\ Stalls Ada\\ Lovelace")
			.await
			.xpect_eq("docs/index.md\ndocs/product.md\ndocs/brand.md\n");
		let today = Date::today();
		fixture.read("docs/index.md").await.xpect_eq(format!(
			"---\ncreated: {today}\nupdated: {today}\nauthors: [Ada Lovelace]\n---\n\n# Acme Stalls\n\n\
			 *TODO(ask): the tagline*\n\n\
			 TODO(ask): the summary paragraph of index, its conclusions in a few sentences.\n\n\
			 ## Mission\n\nTODO(ask): Mission.\n\n## Documents\n\nTODO(ask): Documents.\n"
		));
		fixture
			.read("docs/product.md")
			.await
			.xpect_contains("## Pricing\n\nTODO(ask): Pricing.\n\n```csv price-list\nline,price_ex_gst\n```\n");
		// the scaffold has every shape but the brand document's name, which
		// is still an ask under a decided title
		fixture
			.ok("eval/results")
			.await
			.xpect_contains("12 of 14 checks pass.")
			.xpect_contains("| `structure.index-name-agreed` | fail |");
	}
}
