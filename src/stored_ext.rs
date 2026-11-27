//! Typed documents as stored: a manifest, an outline, a summary or a
//! [`RowTable`](crate::prelude::RowTable) row at one path of a store, pretty JSON with sorted keys and
//! a closing newline, so a change diffs by field.
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
