use crate::prelude::*;
use beet::prelude::*;
use std::collections::BTreeMap;

/// A suite's manifest, `suite.json` at its root: what kind of suite it
/// is, what it builds on, and what the source tags its rows carry mean.
///
/// A suite is a directory in a store with this manifest, addressed by the
/// store's uri, so it may sit beside a workspace, inside one or in a bucket.
/// Its name is lowercase letters, digits and underscores, the `beet_` prefix
/// reserved for suites that are part of the system. Every structured thing
/// in it is a row of a table or a typed document; prose for humans is
/// markdown.
///
/// | Path | Kind | Holds |
/// |---|---|---|
/// | `suite.json` | every | this manifest |
/// | `README.md` | every | what the suite is and provides, for a person |
/// | `outline.json` | document | the [`Outline`] |
/// | `evals/<id>` | any | [`Eval`] rows |
/// | `rubrics/<id>` | any | [`Rubric`] rows |
/// | `render/<rubric>` | reader | [`RenderSpec`] rows |
/// | `actions/<id>` | document | [`CoachAction`] rows |
/// | `claims/<id>` | workspace | [`Claim`] rows |
/// | `assets/` | any | the files the suite provides or was built from, which no table depends on to be checked |
///
/// A table is a [`RowTable`] over the suite's
/// store: one blob per row at `<table>/<key>`, the key being the row's id
/// verbatim with no extension, so `evals/market.competitors-named` holds that
/// eval alone, pretty-printed, and a tool call writing one row touches one
/// object. A table native to its backend, ie a SQLite or DynamoDB store, keeps
/// the same table names and keys.
///
/// Eval ids are global across a workspace, so a rubric cites an eval in any
/// suite by id alone; [`builds_on`](Self::builds_on) names the suites
/// whose evals a suite's rubrics cite.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct SuiteManifest {
	/// The suite's name, the first half of its [`RubricRef`]s.
	pub name: SmolStr,
	/// What the suite is for.
	pub kind: SuiteKind,
	/// The suites whose evals this one's rubrics cite.
	pub builds_on: Vec<SmolStr>,
	/// Every source tag this suite owns, with what it means. A key is a tag
	/// as written, ie `ref:sba`, or a family of tags with its varying parts in
	/// angle brackets, ie `mc:<topic>/<slug>`; a reference to the literature is
	/// a `ref:<slug>` entry naming the work and the idea taken from it.
	pub sources: BTreeMap<SourceTag, String>,
	/// A document suite's outline, ie `outline.json`.
	pub outline: Option<RelPath>,
}

impl SuiteManifest {
	/// The manifest's path at a suite's root.
	pub const FILE: &str = "suite.json";

	/// Whether `name` is a valid suite name: lowercase letters, digits and
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

/// The three kinds of suite.
#[derive(
	Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize,
)]
pub enum SuiteKind {
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

/// A table a suite holds rows of, named by the directory its rows sit in:
/// what `eval/rows` lists, `eval/put` writes a row of and `eval/drop` removes
/// one from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum SuiteTable {
	/// `evals/`, [`Eval`] rows.
	Evals,
	/// `rubrics/`, [`Rubric`] rows.
	Rubrics,
	/// `render/`, a reader suite's [`RenderSpec`] rows.
	Render,
	/// `actions/`, a document suite's [`CoachAction`] rows.
	Actions,
	/// `claims/`, a workspace suite's [`Claim`] rows, its register.
	Claims,
}

impl SuiteTable {
	/// The table's directory, ie `evals`.
	pub fn name(&self) -> SmolStr {
		match self {
			Self::Evals => Eval::table_name(),
			Self::Rubrics => Rubric::table_name(),
			Self::Render => RenderSpec::table_name(),
			Self::Actions => CoachAction::table_name(),
			Self::Claims => Claim::table_name(),
		}
	}
}

impl core::fmt::Display for SuiteTable {
	fn fmt(
		&self,
		formatter: &mut core::fmt::Formatter<'_>,
	) -> core::fmt::Result {
		formatter.write_str(&self.name())
	}
}

/// A suite read: its manifest, the store it lives in, and its tables.
#[derive(Debug, Clone)]
pub struct LoadedSuite {
	/// Its manifest.
	pub manifest: SuiteManifest,
	/// Its store, every path below relative to it.
	pub store: BlobStore,
	/// Where the workspace says it is.
	pub source: SuiteSource,
	/// Its `evals` rows.
	pub evals: Vec<Eval>,
	/// Its `rubrics` rows.
	pub rubrics: Vec<Rubric>,
	/// Its `render` rows.
	pub render: Vec<RenderSpec>,
	/// Its `actions` rows.
	pub actions: Vec<CoachAction>,
	/// Its `claims` rows, a workspace suite's register.
	pub claims: Vec<Claim>,
	/// A document suite's outline.
	pub outline: Option<Outline>,
}

impl LoadedSuite {
	/// Reads the suite at the root of `store`, with every row that could
	/// not be read listed.
	pub async fn load(
		store: BlobStore,
		source: SuiteSource,
	) -> Result<(Self, Vec<String>)> {
		let manifest = stored_ext::read::<SuiteManifest>(
			&store,
			&RelPath::new(SuiteManifest::FILE),
		)
		.await?;
		let mut problems = Vec::new();
		let outline = match &manifest.outline {
			Some(path) => match stored_ext::read::<Outline>(&store, path).await
			{
				Ok(outline) => Some(outline),
				Err(err) => {
					problems.push(format!("{}: {err}", manifest.name));
					None
				}
			},
			None => None,
		};
		let name = manifest.name.clone();
		let suite = Self {
			evals: Self::rows(&store, &name, &mut problems).await?,
			rubrics: Self::rows(&store, &name, &mut problems).await?,
			render: Self::rows(&store, &name, &mut problems).await?,
			actions: Self::rows(&store, &name, &mut problems).await?,
			claims: Self::rows(&store, &name, &mut problems).await?,
			manifest,
			store,
			source,
			outline,
		};
		(suite, problems).xok()
	}

	/// Every row of `T`'s table, the rows that could not be read listed as
	/// the suite's problems.
	async fn rows<T: TableStoreRow + DeserializeOwned>(
		store: &BlobStore,
		suite: &str,
		problems: &mut Vec<String>,
	) -> Result<Vec<T>> {
		let (rows, row_problems) =
			RowTable::<T>::new(store.clone()).all().await?;
		problems.extend(
			row_problems
				.into_iter()
				.map(|problem| format!("{suite}/{problem}")),
		);
		rows.xok()
	}

	/// The render spec of `rubric`.
	pub fn render_spec(&self, rubric: &str) -> Option<&RenderSpec> {
		self.render.iter().find(|spec| spec.rubric == rubric)
	}
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
	/// `<table>/<key>`, so writing one row touches one file, and a suite
	/// scoped out of a larger store keeps its tables beneath its own path.
	#[beet::test]
	async fn one_blob_per_row_on_fs() {
		let dir = TempDir::new().unwrap();
		let store = BlobStore::new(FsStore::new((*dir).clone()))
			.with_subdir("suites/acme".into());
		let evals = RowTable::<Eval>::new(store.clone());
		evals.push(eval("product.origin")).await.unwrap();
		evals.push(eval("market.competitors-named")).await.unwrap();

		store
			.list()
			.await
			.unwrap()
			.xtap(|paths| paths.sort())
			.xpect_eq(vec![
				RelPath::new("evals/market.competitors-named"),
				RelPath::new("evals/product.origin"),
			]);
		fs_ext::read_to_string(dir.join("suites/acme/evals/product.origin"))
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
