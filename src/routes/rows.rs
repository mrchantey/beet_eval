use crate::prelude::*;
use beet::prelude::*;

/// `eval/rows --suite --table [--key]`: a table's rows, or one row by its
/// key, each with its fields, the [`RowsReport`] for a markup `Accept` and
/// the rows in their stored form for a serde one: how a person or an agent
/// reads a register or a table without opening its files, and what a coach
/// sitting is seeded with.
#[action(route = "rows")]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
pub async fn EvalRows(
	/// The suite whose table is read.
	#[param]
	suite: SmolStr,
	/// The table read.
	#[param]
	table: SuiteTable,
	/// One row's key, its id, every row when absent.
	#[param]
	key: Option<SmolStr>,
	cx: ActionContext<Request>,
) -> Result<DataPage<Vec<Value>>> {
	let workspace = LoadedWorkspace::of(&cx.caller).await?;
	let loaded = workspace
		.suite(&suite)
		.ok_or_else(|| bevyhow!("the workspace names no suite {}", suite))?;
	let mut rows = match table {
		SuiteTable::Evals => TableRow::all(&loaded.evals),
		SuiteTable::Rubrics => TableRow::all(&loaded.rubrics),
		SuiteTable::Render => TableRow::all(&loaded.render),
		SuiteTable::Actions => TableRow::all(&loaded.actions),
		SuiteTable::Claims => TableRow::all(&loaded.claims),
	}?;
	let title = format!("{suite}/{table}");
	if let Some(key) = &key {
		rows.retain(|row| &row.key == key);
		if rows.is_empty() {
			return DataPage::new(
				&cx.caller,
				rsx! { <p>{format!("{title}/{key} is no row")}</p> },
				Vec::new(),
			)
			.await?
			.with_status(StatusCode::UNPROCESSABLE_CONTENT)
			.xok();
		}
	}
	let data = rows.iter().map(|row| row.fields.clone()).collect();
	DataPage::new(
		&cx.caller,
		rsx! { <RowsReport title=title rows=rows/> },
		data,
	)
	.await
}

/// One row of a table as [`RowsReport`] shows it: its key and its stored
/// form.
#[derive(Debug, Default, Clone, PartialEq, Reflect)]
pub struct TableRow {
	/// The row's key, its id.
	pub key: SmolStr,
	/// The row as stored, a map of its fields.
	pub fields: Value,
}

impl TableRow {
	/// Every row of a table, in key order.
	fn all<T: TableStoreRow + Serialize>(rows: &[T]) -> Result<Vec<Self>> {
		let mut rows = rows
			.iter()
			.map(|row| {
				Self {
					key: row.key().to_string().into(),
					fields: Value::from_serde(row)?,
				}
				.xok()
			})
			.collect::<Result<Vec<_>>>()?;
		rows.sort_by(|left, right| left.key.cmp(&right.key));
		rows.xok()
	}

	/// A field's value as a line of prose: text as written, a list of texts
	/// one after another, sentences by a space and anything else by a
	/// semicolon, nothing or empty text as `none`, anything else its JSON.
	fn text(value: &Value) -> String {
		match value {
			Value::Str(text) if text.is_empty() => "none".into(),
			Value::Str(text) => text.to_string(),
			Value::Null => "none".into(),
			Value::List(items) if items.is_empty() => "none".into(),
			Value::List(items)
				if items.iter().all(|item| matches!(item, Value::Str(_))) =>
			{
				let texts = items.iter().map(Self::text).collect::<Vec<_>>();
				let sentences =
					texts.iter().all(|text| text.ends_with(['.', '?', '!']));
				match sentences {
					true => texts.join(" "),
					false => texts.join("; "),
				}
			}
			other => other.to_json_string(),
		}
	}
}

/// A table's rows as a person or an agent reads them: a heading per row, its
/// key, and its fields beneath it.
#[template]
pub fn RowsReport(title: String, rows: Vec<TableRow>) -> impl Bundle {
	let count = match rows.len() {
		0 => "no rows".to_string(),
		1 => "1 row".to_string(),
		count => format!("{count} rows"),
	};
	let rows = rows
		.iter()
		.map(|row| {
			let fields = match &row.fields {
				Value::Map(map) => map
					.0
					.iter()
					.map(|(field, value)| {
						rsx! {
							<li><strong>{field.to_string()}</strong>{format!(": {}", TableRow::text(value))}</li>
						}
					})
					.collect::<Vec<_>>(),
				other => vec![rsx! { <li>{TableRow::text(other)}</li> }],
			};
			rsx! {
				<h3>{row.key.to_string()}</h3>
				<ul>{fields}</ul>
			}
		})
		.collect::<Vec<_>>();
	rsx! {
		<h2>{title}</h2>
		<p>{count}</p>
		{rows}
	}
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	/// A table's rows read as a report of each row's fields, one row by its
	/// key, and the rows in their stored form for a serde `Accept`.
	#[beet::test]
	async fn lists_a_tables_rows() {
		let mut fixture = Fixture::new().await;
		fixture
			.ok("eval/rows --suite=acme_biz --table=evals")
			.await
			.xpect_starts_with("## acme_biz/evals\n\n10 rows\n")
			.xpect_contains("### product.origin");
		fixture
			.ok("eval/rows --suite=acme_biz --table=evals --key=product.origin")
			.await
			.xpect_contains("1 row")
			.xpect_contains("- **anchor**: product#idea")
			.xnot()
			.xpect_contains("### brand.voice-defined");
		fixture
			.refused("eval/rows --suite=acme_biz --table=evals --key=nothing")
			.await
			.xpect_contains("acme_biz/evals/nothing is no row");
		let rows = MediaType::Json
			.deserialize::<Vec<Eval>>(
				fixture
					.ok(
						"eval/rows --suite=acme_biz --table=evals --accept=application/json",
					)
					.await
					.as_bytes(),
			)
			.unwrap();
		rows.len().xpect_eq(10);
	}
}
