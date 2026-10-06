use crate::prelude::*;
use beet::prelude::*;
use std::collections::BTreeMap;

/// A package's manifest, `package.json` at its root: what kind of package it
/// is, what it builds on, and what the source tags its rows carry mean.
///
/// A package is a directory in a store with this manifest, addressed by the
/// store's uri, so it may sit beside a workspace, inside one or in a bucket.
/// Its name is lowercase letters, digits and underscores, the `beet_` prefix
/// reserved for packages that are part of the system. Every structured thing
/// in it is a row of a table or a typed document; prose for humans is
/// markdown.
///
/// | Path | Kind | Holds |
/// |---|---|---|
/// | `package.json` | every | this manifest |
/// | `README.md` | every | what the package is and provides, for a person |
/// | `outline.json` | document | the [`Outline`] |
/// | `evals/<id>` | any | [`Eval`] rows |
/// | `rubrics/<id>` | any | [`Rubric`] rows |
/// | `render/<rubric>` | reader | [`RenderSpec`] rows |
/// | `actions/<id>` | document | [`CoachAction`] rows |
/// | `claims/<id>` | workspace | [`Claim`] rows |
/// | `assets/` | any | the files the package provides or was built from, which no table depends on to be checked |
///
/// A table is the json-over-blobs form of [`TableStore`] over the package's
/// store: one blob per row at `<table>/<key>`, the key being the row's id
/// verbatim with no extension, so `evals/market.competitors-named` holds that
/// eval alone, pretty-printed, and a tool call writing one row touches one
/// object. A table native to its backend, ie a SQLite or DynamoDB store, keeps
/// the same table names and keys.
///
/// Eval ids are global across a workspace, so a rubric cites an eval in any
/// package by id alone; [`builds_on`](Self::builds_on) names the packages
/// whose evals a package's rubrics cite.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct PackageManifest {
	/// The package's name, the first half of its [`RubricRef`]s.
	pub name: SmolStr,
	/// What the package is for.
	pub kind: PackageKind,
	/// The packages whose evals this one's rubrics cite.
	pub builds_on: Vec<SmolStr>,
	/// Every source tag this package owns, with what it means. A key is a tag
	/// as written, ie `ref:sba`, or a family of tags with its varying parts in
	/// angle brackets, ie `mc:<topic>/<slug>`; a reference to the literature is
	/// a `ref:<slug>` entry naming the work and the idea taken from it.
	pub sources: BTreeMap<SourceTag, String>,
	/// A document package's outline, ie `outline.json`.
	pub outline: Option<RelPath>,
}

impl PackageManifest {
	/// The manifest's path at a package's root.
	pub const FILE: &str = "package.json";

	/// Whether `name` is a valid package name: lowercase letters, digits and
	/// underscores, starting with a letter.
	pub fn is_name(name: &str) -> bool {
		name.starts_with(|char: char| char.is_ascii_lowercase())
			&& name.chars().all(|char| {
				char.is_ascii_lowercase()
					|| char.is_ascii_digit()
					|| char == '_'
			})
	}
}

/// The three kinds of package.
#[derive(
	Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize,
)]
pub enum PackageKind {
	/// Says what the documentation is: the [`Outline`] naming the
	/// documents and their sections, the evals every document is graded
	/// against, the owner rubric, and the coach's actions. One per workspace.
	Document,
	/// Holds what one outside reader needs: a rubric per form, the assets the
	/// forms come from, and a [`RenderSpec`] per form it renders.
	Reader,
	/// The subject's own: its raw material as assets, its claims register, and
	/// any evals that are its alone.
	Workspace,
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	fn eval(id: &str) -> Eval {
		Eval {
			id: EvalId::parse(id).unwrap(),
			statement: "A statement.".into(),
			levels: Levels::Generic,
			sources: vec![SourceTag::parse("inferred").unwrap()],
			anchor: None,
			note: None,
		}
	}

	/// A table over a filesystem store is one pretty JSON file per row at
	/// `<table>/<key>`, so writing one row touches one file, and a package
	/// scoped out of a larger store keeps its tables beneath its own path.
	#[beet::test]
	async fn one_blob_per_row_on_fs() {
		let dir = TempDir::new().unwrap();
		let store = BlobStore::new(FsStore::new((*dir).clone()))
			.with_subdir("packages/acme".into());
		let evals = TableStore::new(store.clone()).table::<Eval>();
		evals.push(eval("product.origin")).await.unwrap();
		evals.push(eval("market.competitors-named")).await.unwrap();

		BlobStoreProvider::list(&store)
			.await
			.unwrap()
			.xpect_eq(vec![
				RelPath::new("evals/market.competitors-named"),
				RelPath::new("evals/product.origin"),
			]);
		fs_ext::read_to_string(dir.join("packages/acme/evals/product.origin"))
			.unwrap()
			.xpect_starts_with("{\n")
			.xpect_ends_with("}\n")
			.xpect_contains("\n  \"id\": \"product.origin\",\n")
			.xnot()
			.xpect_contains("market.competitors-named");
		evals
			.get("product.origin")
			.await
			.unwrap()
			.xpect_eq(eval("product.origin"));
	}
}
