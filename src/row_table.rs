//! A table as stored: one blob per row, see [`RowTable`].
use crate::prelude::*;
use beet::prelude::*;
use core::marker::PhantomData;

/// The rows of `T`'s table in a [`BlobStore`], one blob per row at
/// `<table>/<key>`, the key being the row's id verbatim with no extension, in
/// the [`stored_ext`] form. Writing one row touches one object and a change
/// diffs by field, which beet's own [`TableStore`] backends, keeping a table
/// in a database or a partition in one file, do not give.
#[derive(Debug, Clone)]
pub struct RowTable<T> {
	store: BlobStore,
	_marker: PhantomData<T>,
}

impl<T: TableStoreRow> RowTable<T> {
	/// `T`'s table in `store`.
	pub fn new(store: BlobStore) -> Self {
		Self {
			store,
			_marker: PhantomData,
		}
	}

	/// Writes `row` at its key, replacing any row there.
	pub async fn push(&self, row: T) -> Result {
		stored_ext::write(
			&self.store,
			&Self::path(&row.key().to_string()),
			&row,
		)
		.await
	}

	/// The row at `key`, an error if there is none or it does not parse.
	pub async fn get(&self, key: &str) -> Result<T> {
		stored_ext::read(&self.store, &Self::path(key)).await
	}

	/// Whether a row is at `key`.
	pub async fn exists(&self, key: &str) -> Result<bool> {
		self.store.exists(&Self::path(key)).await
	}

	/// Removes the row at `key`.
	pub async fn remove(&self, key: &str) -> Result {
		self.store.remove(&Self::path(key)).await
	}

	/// Every row in key order, with the reason for each row that could not
	/// be read: one that does not parse, or whose key is not its id. An
	/// absent table is empty.
	pub async fn all(&self) -> Result<(Vec<T>, Vec<String>)> {
		let table = T::table_name();
		let scoped = self.store.with_subdir(RelPath::new(table.as_str()));
		if !scoped.store_exists().await.unwrap_or(false) {
			return (Vec::new(), Vec::new()).xok();
		}
		let mut keys = scoped.list().await?;
		keys.sort();
		let mut rows = Vec::new();
		let mut problems = Vec::new();
		for key in keys {
			let path = Self::path(key.as_str());
			match stored_ext::read::<T>(&self.store, &path).await {
				Ok(row) if row.key().to_string() == key.as_str() => {
					rows.push(row)
				}
				Ok(row) => problems.push(format!(
					"{path}: the row's id is `{}`, not its key",
					row.key()
				)),
				Err(err) => problems.push(err.to_string()),
			}
		}
		(rows, problems).xok()
	}

	/// The blob path of the row at `key`.
	fn path(key: &str) -> RelPath {
		RelPath::new(format!("{}/{key}", T::table_name()))
	}
}
