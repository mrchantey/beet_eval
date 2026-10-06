//! The renderer: building `docs/` into one of a reader package's forms, the
//! reader's own Word file or workbook, through beet's Office support.
//!
//! A build has a deterministic half, the projection and the filling, and a
//! judged half, the writing of each cell in the form's voice; the structural
//! lines of the form's rubric verify the result. Nothing is written into a
//! package's template: a build copies it to `dist/<package>/` and fills the
//! copy, and the build directory is rebuilt at will and never kept.
//!
//! # The steps
//!
//! 1. `eval/project <package>/<rubric>` writes the brief,
//!    `results/projections/<package>-<rubric>.md`: every heading of the
//!    rubric, the evals it cites, where each is anchored, what is written
//!    there, and the heading's structural lines. The rubric read through the
//!    anchors is the mapping, and there is no other.
//! 2. The builder writes the [`FillSpec`] from the brief and the template's
//!    cells dump.
//! 3. `eval/build <package>/<rubric>` copies the template the [`RenderSpec`]
//!    names to `dist/<package>/<output>`, applies the fill spec, and dumps the
//!    result's cells.
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
//! A [`CellRef`] names what the cells dump of a form prints: every table cell
//! of a Word file as `t<table>r<row>c<cell> | text`, every unlocked cell of a
//! workbook as `<sheet>!<A1> | value`. The dump is regenerated from the
//! template whenever a builder needs it and never stored, so it cannot drift
//! from the file it describes.
//!
//! # The fill spec
//!
//! | Op | Target | Does |
//! |---|---|---|
//! | [`Set`](FillOp::Set) | a Word cell | replaces the cell's paragraphs with one per line, in the cell's own style |
//! | [`Set`](FillOp::Set) | a workbook cell | writes the value, a number when it parses as one, into an unlocked cell; a locked cell or a formula is refused |
//! | [`Append`](FillOp::Append) | a Word cell | adds paragraphs after the cell's own |
//! | [`Check`](FillOp::Check) | a label | ticks the checkbox control beside it: glyph ticked, bold, highlighted |
//! | [`Delete`](FillOp::Delete) | a text | removes every paragraph carrying it; a cell keeps one empty paragraph |
//! | [`Replace`](FillOp::Replace) | a text | replaces it inside the runs that carry it, keeping their formatting |
mod fill;
mod render_spec;
pub use fill::*;
pub use render_spec::*;
