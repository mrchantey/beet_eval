//! The renderer: building `docs/` into one of a reader package's forms, the
//! reader's own Word file or workbook. The builder, an agent, first writes
//! the form's answers as markdown, which a person reviews and approves; then
//! it writes the approved answers into the form's own XML, so the result
//! comes back looking as its author made it: beet reads a form into its one
//! document tree to list its cells, and never writes one.
//!
//! A build has a deterministic half, the checking of the answers against the
//! form's cells, the unpacking, packing and listing, and a judged half, the
//! writing of each answer in the form's voice and its placing in the form's
//! XML; the structural lines of the form's rubric verify both. Nothing is
//! written into the reader's blank form: a build unpacks a copy into the
//! build directory and edits the copy, and the build directory is rebuilt at
//! will; the approved answers are kept in the results.
//!
//! # The steps
//!
//! 1. `eval/project <package>/<rubric>` writes the brief,
//!    `results/projections/<package>-<rubric>.md`: every heading of the
//!    rubric, the evals it cites, where each is anchored, what is written
//!    there, and the heading's structural lines. The rubric read through the
//!    anchors is the mapping, and there is no other.
//! 2. The builder writes the form's [`AnswerSheet`] to
//!    `dist/<package>/<rubric>.answers.md`, one `## <cell>` section per cell
//!    it answers, the cells listed by `eval/cells <package>/<rubric>`, and an
//!    `## Edits` list of what the form needs beyond its cells' text.
//! 3. `eval/answers <package>/<rubric>` refuses a sheet answering a cell the
//!    form lacks or leaving one it names empty, and otherwise sets it out for
//!    review: each answer under its cell's blank text, the empty cells left
//!    unanswered, the edits, and the rubric's structural lines, each read
//!    against the sheet. A person reviews it and approves it with
//!    `eval/approve <package>/<rubric> --by=<who>` ([`AnswerApproval`]),
//!    which no role can call, and which lapses when the sheet changes.
//! 4. `eval/build <package>/<rubric>`, refused until the sheet is approved as
//!    it stands, unpacks the blank form the [`RenderSpec`] names into
//!    `dist/<package>/<rubric>/`, one file per part of its package
//!    ([`FormBuild`]), the first time or with `--fresh`; then zips those
//!    files into `dist/<package>/<output>`, reads it, and writes its
//!    [`CellsReport`] to `dist/<package>/<rubric>.cells.md`, which the triage
//!    reads as built.
//! 5. The builder writes the approved answers into the unpacked parts with
//!    the store's text tools, ie `word/document.xml`: a cell's paragraphs in
//!    the cell's own paragraph and run properties, a checkbox's
//!    `w14:checked` and glyph ticked, a red instruction sentence's paragraph
//!    removed. A workbook's edited formulas are left for the spreadsheet to
//!    calculate, its `calcPr` set to `fullCalcOnLoad`.
//! 6. `eval/build` again, which refuses parts that no longer read, and the
//!    cells read against the approved sheet and the rubric's structural lines
//!    one by one, edits and rebuilds until every line holds.
//! 7. The file sent is copied as sent into the workspace package's
//!    `assets/submissions/`, with the reader's feedback filed beside it when it
//!    comes.
//!
//! # Cells
//!
//! A [`CellRef`] names a cell by its place, `<table>/<x>,<y>`, its slot in
//! the table's grid as beet's `TableGrid` counts it, from 0, column then
//! row: a table outside a worksheet is `t<n>` by its place among the
//! document's tables, ie `t0/1,0`, and a worksheet's is its
//! `data-worksheet`, ie `Start Here/3,2`. A form's cells are listed
//! whenever a builder needs them and never stored, so they cannot drift
//! from the file they describe.
mod answer_approval;
mod answer_sheet;
mod cells_report;
mod form_build;
mod form_cell;
mod render_spec;
pub use answer_approval::*;
pub use answer_sheet::*;
pub use cells_report::*;
pub use form_build::*;
pub use form_cell::*;
pub use render_spec::*;
