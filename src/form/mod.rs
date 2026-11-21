//! The renderer: building `docs/` into one of a reader package's forms, the
//! reader's own Word file or workbook. The builder, an agent, writes the
//! answers into the form's own XML, so the result comes back looking as its
//! author made it: beet reads a form into its one document tree to list its
//! cells, and never writes one.
//!
//! A build has a deterministic half, the unpacking, packing and listing, and
//! a judged half, the writing of each cell in the form's voice; the
//! structural lines of the form's rubric verify the result. Nothing is
//! written into the reader's blank form: a build unpacks a copy into the
//! build directory and edits the copy, and the build directory is rebuilt at
//! will and never kept.
//!
//! # The steps
//!
//! 1. `eval/project <package>/<rubric>` writes the brief,
//!    `results/projections/<package>-<rubric>.md`: every heading of the
//!    rubric, the evals it cites, where each is anchored, what is written
//!    there, and the heading's structural lines. The rubric read through the
//!    anchors is the mapping, and there is no other.
//! 2. `eval/build <package>/<rubric>` unpacks the blank form the
//!    [`RenderSpec`] names into `dist/<package>/<rubric>/`, one file per part
//!    of its package ([`FormBuild`]), the first time or with `--fresh`; then
//!    zips those files into `dist/<package>/<output>`, reads it, and writes
//!    its [`CellsReport`] to `dist/<package>/<rubric>.cells.md`, which the
//!    triage reads as built. `eval/cells <package>/<rubric>` lists the blank
//!    form's cells without building.
//! 3. The builder edits the unpacked parts with the store's text tools, ie
//!    `word/document.xml`: a cell's paragraphs written in the cell's own
//!    paragraph and run properties, a checkbox's `w14:checked` and glyph
//!    ticked, a red instruction sentence's paragraph removed. A workbook's
//!    edited formulas are left for the spreadsheet to calculate, its
//!    `calcPr` set to `fullCalcOnLoad`.
//! 4. `eval/build` again, which refuses parts that no longer read, and the
//!    cells read against the rubric's structural lines one by one, edits and
//!    rebuilds until every line holds.
//! 5. The file sent is copied as sent into the workspace package's
//!    `assets/submissions/`, with the reader's feedback filed beside it when it
//!    comes.
//!
//! # Cells
//!
//! A [`CellRef`] names a cell by its place, as beet's `TableGrid` counts a
//! table's slots and a workbook's `A1Reference` spells one: a table's cell
//! as `t<table>/<A1>`, its table's place among the document's tables from
//! 0, ie `t0/B1`, and a workbook's as `<sheet>!<A1>`, its worksheet's
//! `article data-worksheet`. A form's cells are listed whenever a builder
//! needs them and never stored, so they cannot drift from the file they
//! describe.
mod cells_report;
mod form_build;
mod form_cell;
mod render_spec;
pub use cells_report::*;
pub use form_build::*;
pub use form_cell::*;
pub use render_spec::*;
