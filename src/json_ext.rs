//! Typed documents as stored: a manifest, an outline, a summary or a fill
//! spec at one path of a store, written in the same form a table row is,
//! pretty JSON with sorted keys and a closing newline, so a change diffs by
//! field.
use beet::prelude::*;

/// Reads the typed document at `path`.
pub async fn read<T: DeserializeOwned>(
	store: &BlobStore,
	path: &RelPath,
) -> Result<T> {
	let bytes = store
		.get(path)
		.await
		.map_err(|err| bevyhow!("cannot read `{path}`: {err}"))?;
	MediaType::Json
		.deserialize::<T>(&bytes)
		.map_err(|err| bevyhow!("`{path}` does not parse: {err}"))
}

/// Reads the typed document at `path`, `None` when there is none.
pub async fn read_optional<T: DeserializeOwned>(
	store: &BlobStore,
	path: &RelPath,
) -> Result<Option<T>> {
	match store.exists(path).await? {
		true => read(store, path).await.map(Some),
		false => Ok(None),
	}
}

/// Writes `value` at `path` in the stored form.
pub async fn write<T: Serialize>(
	store: &BlobStore,
	path: &RelPath,
	value: &T,
) -> Result {
	store.insert(path, to_string(value)?).await
}

/// `value` in the stored form.
pub fn to_string<T: Serialize>(value: &T) -> Result<String> {
	// through `Value`, whose maps are sorted
	let value = Value::from_serde(value)?;
	let bytes = MediaType::Json
		.serialize_with_options(&value, SerializeOptions { pretty: true })?;
	format!("{}\n", String::from_utf8(bytes)?).xok()
}

/// Every row of `T`'s table under `store` in key order, with the reason for
/// each row that could not be read: one that does not parse, or whose key is
/// not its id. An absent table is empty.
pub async fn rows<T: TableStoreRow + DeserializeOwned>(
	store: &BlobStore,
) -> Result<(Vec<T>, Vec<String>)> {
	let table = T::table_name();
	let scoped = store.with_subdir(RelPath::new(table.as_str()));
	if !scoped.store_exists().await.unwrap_or(false) {
		return (Vec::new(), Vec::new()).xok();
	}
	let mut keys = scoped.list().await?;
	keys.sort();
	let mut rows = Vec::new();
	let mut problems = Vec::new();
	for key in keys {
		let path = RelPath::new(format!("{table}/{key}"));
		match read::<T>(store, &path).await {
			Ok(row) if row.key().to_string() == key.as_str() => rows.push(row),
			Ok(row) => problems.push(format!(
				"{path}: the row's id is `{}`, not its key",
				row.key()
			)),
			Err(err) => problems.push(err.to_string()),
		}
	}
	(rows, problems).xok()
}
