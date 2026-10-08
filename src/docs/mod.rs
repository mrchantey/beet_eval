//! The first subject: the documents under a workspace's documents directory,
//! prose for people, each a markdown file or a Word file read the same way. A
//! document package's [`Outline`] says which documents exist and what their
//! sections are; this is what any one of them looks like, so a reader, a
//! grader and a renderer can rely on it. Read, a document is a tree in a
//! [`DocumentSet`], and its frontmatter is beet's
//! [`PageMeta`](beet::prelude::PageMeta).
//!
//! # One file or a directory
//!
//! A document starts as one file, `market.md`, and is promoted to a directory
//! when a section outgrows it: `market/index.md` with `market/customers.md`
//! beside it, the parent keeping its summary, every `##` heading, and under the
//! promoted heading a summary and a link to the child. Nothing else changes and
//! every address stays the same. A child is reached through its parent, so the
//! set stays a handful at the top however deep it goes.
//!
//! # Anatomy
//!
//! ```md
//! ---
//! created: 2026-10-02
//! updated: 2026-10-02
//! authors: [Ada Lovelace]
//! ---
//!
//! # Product
//!
//! One paragraph a reader who stops here can act on: what this document
//! establishes, as conclusions rather than as a list of its sections.
//!
//! ## Idea
//!
//! Prose, with a `Sources:` line at the end of the section where its facts came
//! from somewhere.
//! ```
//!
//! 1. **Frontmatter**: `created`, `updated` and `authors`, the
//!    [`DocumentSet::META_KEYS`] of beet's `PageMeta`, and nothing else, since
//!    the title and the summary are the body's. `updated` moves on every
//!    substantive edit, and is what a reader checks before trusting a figure.
//! 2. **Title**: the level one heading is the document's name. For the index it
//!    is the subject's name, with the tagline as the emphasised line beneath
//!    it, `*like this*`. The index is the source of truth for both, and the
//!    document that owns them in detail is checked for agreement with it.
//! 3. **Summary**: one paragraph before the first heading, the document's
//!    conclusions, so the index can carry them and a reader in a hurry needs
//!    nothing else.
//! 4. **Sections**: `##` headings in the order the outline fixes, every one
//!    present even when its body is one sentence saying why it does not apply.
//!    A [`DocumentSection`] is an anchor, and anchors do not move. `###` is
//!    free below.
//! 5. **Sources**: a section may end with a `Sources:` line naming where its
//!    facts came from: an interview, a quote, a register, a page. A number
//!    without a source is an estimate and says so.
//! 6. **Asks**: an open question for the owner sits inline where its answer
//!    belongs and nowhere else, an [`Ask`]: `TODO(ask fact: ...)` when a source
//!    could answer it, `TODO(ask decision: ...)` when only the owner can, a
//!    bare `TODO(ask: ...)` being a fact. The question may instead follow the
//!    parentheses to the end of the line, `TODO(ask): the tagline`, which is
//!    what a scaffold writes. A section with one open is graded at most 1 on
//!    every eval anchored there.
//!
//! # Writing rules
//!
//! First person plural or the subject's name, present tense, one idea per
//! sentence. A sentence that could be said of any subject is deleted. A claim
//! carries its evidence or its source. A number lives in a data block, with the
//! prose saying what it means. A reader who demands another voice gets it at
//! build time, never here.
//!
//! # Data blocks
//!
//! Anything tabular or numeric that another document or a renderer consumes is
//! a [`DataBlock`], a fence named in its info string, `csv` for a table and
//! `json` only where the data nests:
//!
//! ````md
//! ```csv price-list
//! line,unit,price_ex_gst,gst,direct_cost,hours_per_unit
//! Day of engineering,day,1200,yes,0,8
//! ```
//! ````
//!
//! A name is defined once across the documents, in the document that owns the
//! fact, and every other document links to it rather than copying it. The
//! first row is the header and the columns are the outline's, each a
//! [`Column`] that may be typed `num`, `month` (`2026-10`) or `date`
//! (`2026-10-02`). An item that does not apply is a 0 with the reason in the
//! prose; money is whole dollars unless the outline says otherwise.
//!
//! # Addressing
//!
//! A place in the documents is an [`Address`](crate::prelude::Address),
//! `document` or `document#section`, the section half being
//! [`Address::slug`](crate::prelude::Address::slug) of its heading: `legal#risk-register`,
//! `finance#owners-finances`. An eval's
//! anchor, a block's home, a claim's block and a grade's evidence are
//! addresses, and promotion keeps every one stable.
//!
//! # One definition
//!
//! A fact is written in exactly one place and linked from every other. The
//! index is the one exception and the only summary layer: it may restate a
//! child's conclusion in a line, never carry a fact the child lacks, and every
//! number it shows appears in a child's data block. Agreement between two
//! renderings is then a property of the pipeline rather than something to
//! proofread.
//!
//! # Reading
//!
//! Every file of the documents directory is parsed once by beet's media
//! parse into a document root of a [`DocumentSet`], a markdown file and a
//! `.docx` alike, since both read into the same HTML terms. What a document
//! is, its title, tagline, summary, sections, data blocks and asks, is the
//! traversals of [`ProseQuery`] over that tree, so a heading inside a
//! fence is no heading, a block may sit in a list, and a Word file's heading
//! styles, emphasis and tables read as a markdown file's do.
//!
//! # Checks
//!
//! The kinds deciding a document's shape are routes under `check/`, their
//! actions and params in [`checks`]; an outline generates the evals restating
//! it, and a package writes the rest as rows.
mod ask;
pub mod checks;
mod data_block;
mod document_set;
mod outline;
mod prose_query;
pub use ask::*;
pub use checks::*;
pub use data_block::*;
pub use document_set::*;
pub use outline::*;
pub use prose_query::*;
