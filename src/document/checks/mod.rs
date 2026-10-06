//! The document check kinds, each a route under `check/` whose params type is
//! here. A document is named without its extension, `brand` for `brand.md` or
//! `brand/index.md` under the workspace's documents directory, so promotion
//! never breaks a check.
//!
//! | Kind | Params | Passes when |
//! |---|---|---|
//! | `document` | [`DocumentCheckParams`] | the document exists as a file or a directory with an index |
//! | `frontmatter` | [`FrontmatterCheckParams`] | every document resolves and its frontmatter carries every key with a value |
//! | `h1` | [`H1CheckParams`] | the first non-blank line after the frontmatter is a level one heading |
//! | `tagline` | [`TaglineCheckParams`] | one emphasised line stands alone beneath the title |
//! | `summary` | [`SummaryCheckParams`] | every document has a paragraph between its title, or tagline, and its first `##` |
//! | `agrees` | [`AgreesCheckParams`] | the document's title or tagline appears verbatim in the section that owns it, or both carry the same open ask |
//! | `asks` | [`AsksCheckParams`] | no ask is open anywhere under the documents directory |
//! | `sections` | [`SectionsCheckParams`] | the document's `##` headings are exactly these, in order |
//! | `block` | [`BlockCheckParams`] | the named `csv` block is defined once, under its section, with exactly these columns, each typed cell of its type |
//!
//! A document package's
//! [`DocumentTemplate`](crate::prelude::DocumentTemplate) generates the
//! `document`, `sections` and `block` evals of its documents and blocks and
//! the `frontmatter` and `summary` evals over the set; the package writes the
//! rest as rows. When a template is present, `eval/check` also refuses an
//! anchor, a block's section or an `agrees` section naming a document or a
//! section the template does not declare, so the template's sections stay the
//! one list of addresses.
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
