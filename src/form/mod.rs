//! The renderer: building `docs/` into one of a reader package's forms, the
//! reader's own Word file or workbook, through beet's Office support: a build
//! parses the blank form into beet's one document tree, applies the fill
//! spec's operations as edits in HTML terms, and renders the tree back in the
//! form's own format.
//!
//! A build has a deterministic half, the projection and the filling, and a
//! judged half, the writing of each cell in the form's voice; the structural
//! lines of the form's rubric verify the result. Nothing is written into the
//! reader's blank form: a build copies it to `dist/<package>/` and fills the
//! copy, and the build directory is rebuilt at will and never kept.
//!
//! # The steps
//!
//! 1. `eval/project <package>/<rubric>` writes the brief,
//!    `results/projections/<package>-<rubric>.md`: every heading of the
//!    rubric, the evals it cites, where each is anchored, what is written
//!    there, and the heading's structural lines. The rubric read through the
//!    anchors is the mapping, and there is no other.
//! 2. The builder writes the [`FillSpec`] from the brief and the blank form's
//!    cells, which `eval/cells <package>/<rubric>` lists.
//! 3. `eval/build <package>/<rubric>` parses the blank form the [`RenderSpec`]
//!    names, applies the fill spec, answering a line per op and a warning for
//!    one that matched nothing, renders the result to
//!    `dist/<package>/<output>`, and writes its [`CellsReport`] to
//!    `dist/<package>/<rubric>.cells.md`, which the triage reads as built.
//! 4. The builder reads the dump against the rubric's structural lines one by
//!    one, fixes the spec and rebuilds until every line holds. A workbook's
//!    computed tabs are read from a spreadsheet application, since the filler
//!    keeps formulas but does not calculate them.
//! 5. The file sent is copied as sent into the workspace package's
//!    `assets/submissions/`, with the reader's feedback filed beside it when it
//!    comes.
//!
//! # Cells
//!
//! A [`CellRef`] names a cell as beet addresses it in any document with
//! tables: every table cell as `t<table>r<row>c<cell>`, every unlocked cell
//! of a workbook as `<sheet>!<A1>`, beet's `TableCellAddress` and
//! `SheetCellAddress`. A blank form's cells are listed whenever a builder
//! needs them and never stored, so they cannot drift from the file they
//! describe.
//!
//! # The fill spec
//!
//! | Op | Target | Does |
//! |---|---|---|
//! | [`Set`](FillOp::Set) | a table cell | replaces the cell's paragraphs with one per line, in the cell's own style |
//! | [`Set`](FillOp::Set) | a workbook cell | writes the value, a number when it parses as one, into an unlocked cell, a merged cell's range's first; a locked cell or a formula fails the build |
//! | [`Append`](FillOp::Append) | a table cell | adds paragraphs after the cell's own |
//! | [`Check`](FillOp::Check) | a label | checks the box beside it; a Word form's glyph is ticked, bold and highlighted |
//! | [`Delete`](FillOp::Delete) | a text | removes every paragraph carrying it; a cell keeps one empty paragraph |
//! | [`Replace`](FillOp::Replace) | a text | replaces it inside the words that carry it, keeping their look |
//!
//! Each op is one of beet's edits, `SetText`, `AppendText`, `CheckBox`,
//! `RemoveParagraphs` and `ReplaceText`, so one fill spec fills a markdown
//! form and a Word form alike. A workbook build is marked to recalculate on
//! open, since the filler keeps formulas but computes none.
mod cells_report;
mod fill;
mod render_spec;
pub use cells_report::*;
pub use fill::*;
pub use render_spec::*;
