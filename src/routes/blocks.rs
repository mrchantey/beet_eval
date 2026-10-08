use super::*;
use crate::prelude::*;
use beet::prelude::*;

/// Request params for [`EvalBlocks`], surfaced in `--help`.
#[derive(Reflect)]
struct BlocksParams {
	/// The block to print, the `:name` path segment.
	name: Option<SmolStr>,
}

/// `eval/blocks [<name>]`: lists the named data blocks under the documents
/// directory, the [`BlocksReport`] for a markup `Accept` and every block in
/// full, its rows parsed, for a serde one; or prints one block's body as
/// text, for a calculation or a renderer.
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
	let mut documents = DocumentSet::load(
		&store.with_subdir(manifest.docs.clone()),
		manifest.docs,
	)
	.await?;
	let blocks = documents.data_blocks();
	if let Some(name) = params.name {
		return match blocks.iter().find(|(_, block)| block.name == name) {
			Some((_, block)) => Response::ok_text(format!("{}\n", block.text)),
			None => refusal(format!(
				"no block named {name} under {}/\n",
				documents.dir()
			)),
		}
		.xok();
	}
	let dump = BlockDump::new(&documents, &blocks);
	let report = rsx! { <BlocksReport dump=dump.clone() dir=documents.dir().to_string()/> };
	DataPage::new(&cx.caller, report, dump)
		.await?
		.into_response_with_request_parts(
			cx.caller.clone(),
			cx.input.parts().clone(),
		)
		.await
}

/// Every named block, by name in the order they are met: its file, line,
/// section and format, and its rows or its JSON.
#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct BlockDump(Vec<(SmolStr, BlockEntry)>);

/// One block as the dump carries it.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize)]
pub struct BlockEntry {
	file: String,
	line: u32,
	section: Option<SmolStr>,
	format: SmolStr,
	header: Vec<String>,
	rows: Vec<Vec<String>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	json: Option<Value>,
}

impl BlockDump {
	fn new(
		documents: &DocumentSet,
		blocks: &[(DocumentFile, DataBlock)],
	) -> Self {
		let mut entries = Vec::<(SmolStr, BlockEntry)>::new();
		for (document, block) in blocks {
			let entry = BlockEntry {
				file: documents.path_of(document),
				line: block.line,
				section: block.section.clone(),
				format: block.format.word().into(),
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
			match entries.iter_mut().find(|(name, _)| *name == block.name) {
				Some((_, existing)) => *existing = entry,
				None => entries.push((block.name.clone(), entry)),
			}
		}
		Self(entries)
	}
}

/// The dump is a map from name to block, in its own order.
impl Serialize for BlockDump {
	fn serialize<S: serde::Serializer>(
		&self,
		serializer: S,
	) -> Result<S::Ok, S::Error> {
		serializer.collect_map(self.0.iter().map(|(name, entry)| (name, entry)))
	}
}

/// The blocks as a table: each block's name, format, where it sits and its
/// size.
#[template]
pub fn BlocksReport(
	#[prop(required)] dump: BlockDump,
	dir: String,
) -> impl Bundle {
	let rows = dump
		.0
		.iter()
		.map(|(name, entry)| {
			let section = entry
				.section
				.as_ref()
				.map(|section| format!("#{section}"))
				.unwrap_or_default();
			let size = match entry.json {
				Some(_) => "json".to_string(),
				None => format!("{} rows", entry.rows.len()),
			};
			rsx! {
				<tr>
					<td>{name.to_string()}</td>
					<td>{entry.format.to_string()}</td>
					<td>{format!("{}:{}{section}", entry.file, entry.line)}</td>
					<td>{size}</td>
				</tr>
			}
		})
		.collect::<Vec<_>>();
	match rows.is_empty() {
		true => rsx! { <p>{format!("No named blocks under {dir}/.")}</p> }
			.any_bundle(),
		false => rsx! {
			<table>
				<tr><th>"Block"</th><th>"Format"</th><th>"At"</th><th>"Size"</th></tr>
				{rows}
			</table>
		}
		.any_bundle(),
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
			.xpect_eq("| Block | Format | At | Size |\n|---|---|---|---|\n| price-list | csv | docs/product.md:19#pricing | 2 rows |\n");
		fixture
			.ok("eval/blocks price-list")
			.await
			.xpect_eq("line,price_ex_gst\nSmall stall,120\nLarge stall,180\n");
		fixture
			.ok("eval/blocks --accept=application/json")
			.await
			.xpect_starts_with("{\n  \"price-list\": {\n    \"file\": \"docs/product.md\",\n    \"line\": 19,")
			.xpect_contains("\"rows\": [\n      [\n        \"Small stall\",\n        \"120\"\n      ],");
		fixture
			.refused("eval/blocks nothing")
			.await
			.xpect_contains("no block named nothing");
	}
}
