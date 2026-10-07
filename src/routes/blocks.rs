use super::*;
use crate::prelude::*;
use beet::prelude::*;

/// Request params for [`EvalBlocks`], surfaced in `--help`.
#[derive(Reflect)]
struct BlocksParams {
	/// The block to print, the `:name` path segment.
	name: Option<SmolStr>,
	/// Print every block as JSON.
	all: bool,
}

/// `eval/blocks [<name> | --all]`: lists the named data blocks under the
/// documents directory, prints one block's body for a calculation or a
/// renderer, or prints them all as JSON.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
#[require(
	PathPartial = PathPartial::new("blocks/:name?"),
	ParamsPartial = ParamsPartial::new::<BlocksParams>()
)]
pub async fn EvalBlocks(cx: ActionContext<Request>) -> Result<Response> {
	let params = cx.input.parse_params::<BlocksParams>()?;
	let store = LoadedWorkspace::store_of(&cx.caller).await?;
	let manifest = Workspace::read(&store).await?;
	let documents = DocumentSet::load(
		&store.with_subdir(manifest.docs.clone()),
		manifest.docs,
	)
	.await?;
	let blocks = documents.blocks();
	if params.all {
		return Response::ok_text(format!(
			"{}\n",
			BlockDump::json(&documents, &blocks)?
		))
		.xok();
	}
	match params.name {
		Some(name) => match blocks.iter().find(|(_, block)| block.name == name)
		{
			Some((_, block)) => Response::ok_text(format!("{}\n", block.text)),
			None => refusal(format!(
				"no block named {name} under {}/\n",
				documents.dir()
			)),
		},
		None if blocks.is_empty() => Response::ok_text(format!(
			"no named blocks under {}/\n",
			documents.dir()
		)),
		None => Response::ok_text(
			blocks
				.iter()
				.map(|(document, block)| {
					let section = block
						.section
						.as_ref()
						.map(|section| format!("#{section}"))
						.unwrap_or_default();
					let size = match block.format {
						BlockFormat::Csv => {
							format!("{} rows", block.rows.len())
						}
						BlockFormat::Json => "json".into(),
					};
					format!(
						"{}\t{}\t{}:{}{section}\t{size}\n",
						block.name,
						block.format.word(),
						documents.path_of(document),
						block.line
					)
				})
				.collect::<String>(),
		),
	}
	.xok()
}

/// The blocks as one JSON object by name, in the order they are met.
struct BlockDump;

impl BlockDump {
	fn json(
		documents: &DocumentSet,
		blocks: &[(&MarkdownDocument, &DataBlock)],
	) -> Result<String> {
		let mut entries = Vec::<(&str, BlockJson)>::new();
		for (document, block) in blocks {
			let entry = BlockJson {
				file: documents.path_of(document),
				line: block.line,
				section: block.section.clone(),
				format: block.format.word(),
				header: block.header.clone(),
				rows: block.rows.clone(),
				json: match block.format {
					BlockFormat::Json => Some(
						MediaType::Json
							.deserialize::<Value>(block.text.as_bytes())
							.unwrap_or(Value::Null),
					),
					BlockFormat::Csv => None,
				},
			};
			// a name defined twice keeps its first place and its last body
			match entries
				.iter_mut()
				.find(|(name, _)| *name == block.name.as_str())
			{
				Some((_, existing)) => *existing = entry,
				None => entries.push((block.name.as_str(), entry)),
			}
		}
		let bytes = MediaType::Json
			.serialize_with_options(&OrderedMap(entries), SerializeOptions {
				pretty: true,
			})?;
		String::from_utf8(bytes)?.xok()
	}
}

/// One block as the dump prints it.
#[derive(Serialize)]
struct BlockJson {
	file: String,
	line: u32,
	section: Option<SmolStr>,
	format: &'static str,
	header: Vec<String>,
	rows: Vec<Vec<String>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	json: Option<Value>,
}

/// Entries serialized as a map in their own order.
struct OrderedMap<'a>(Vec<(&'a str, BlockJson)>);

impl Serialize for OrderedMap<'_> {
	fn serialize<S: serde::Serializer>(
		&self,
		serializer: S,
	) -> Result<S::Ok, S::Error> {
		serializer.collect_map(self.0.iter().map(|(name, entry)| (name, entry)))
	}
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	#[beet::test]
	async fn lists_prints_and_dumps_blocks() {
		let mut fixture = Fixture::new().await;
		fixture
			.ok("eval/blocks")
			.await
			.xpect_eq("price-list\tcsv\tdocs/product.md:19#pricing\t2 rows\n");
		fixture
			.ok("eval/blocks price-list")
			.await
			.xpect_eq("line,price_ex_gst\nSmall stall,120\nLarge stall,180\n");
		fixture
			.ok("eval/blocks --all")
			.await
			.xpect_starts_with("{\n  \"price-list\": {\n    \"file\": \"docs/product.md\",\n    \"line\": 19,")
			.xpect_contains("\"rows\": [\n      [\n        \"Small stall\",\n        \"120\"\n      ],");
		fixture
			.refused("eval/blocks nothing")
			.await
			.xpect_contains("no block named nothing");
	}
}
