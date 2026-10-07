---
name: build-form
description: >
  Build one of a reader package's forms from a beet_eval workspace's docs/: project the
  rubric, write the fill spec, run the build into dist/, and verify the result against the
  rubric's structural lines. Use when asked to build, render, fill, export or prepare a
  submission document, a plan, a workbook or any form a package's render table names.
---

# build-form

beet_eval's `src/form/mod.rs` (`~/me/beet_eval`) is the design: the steps, the cell addresses and the fill spec's operations. Read it first, then the form's render spec, the reader package's `render/<rubric>` row, whose notes say what this form needs beyond text, and the form's rubric, `rubrics/<rubric>`, whose structural lines are the test. A blank form is never edited; the build copies it.

## Before building

1. `just eval results`. The documents the form draws on should be graded and at the level the rubric cites; a form built from level 1 text is a level 1 form. Build anyway when the owner asks, and say so in the report.
2. `just eval project <package>/<rubric>` for the brief, `results/projections/<package>-<rubric>.md`: every heading of the form, the evals it cites, where each is anchored and what is written there, with the heading's structural lines.
3. `just eval cells <package>/<rubric>` for the blank form's cells, one `| address | text |` row each; every fill target is an address in it. To read the form as a whole, its prompts, red instructions and highlighted placeholders included, `just eval view <path to the form> --accept=text/markdown`: its tables are headed `<!-- t<n> -->` with the numbers the cells dump uses, and what the form signals by look stays as `<mark>` and `<span style>`.

## Writing the fill spec

4. `dist/<package>/<rubric>.fill.json`, a `FillSpec`, its shape in beet_eval's `README.md`. Work heading by heading through the brief. For each fill-in cell, a `Set` or an `Append`: the text is the anchored section of `docs/` rewritten for this reader, in the form's voice and length, every number as it stands in the data blocks, nothing invented. A prompt that does not apply gets the reader's accepted wording, never a blank.
5. Then the form's own demands: boxes ticked with `Check` by their label, red instruction sentences gone with `Delete`, placeholders filled with `Replace`, options that do not apply deleted. The render spec's notes list which of these this form has.
6. For a workbook, `Set` only unlocked cells, from `just eval blocks --all`; the render spec's notes say which block feeds which table.

## Building and verifying

7. `just eval build <package>/<rubric>`. Read its answer for `WARNING op <n>` lines, a label or a text the filler did not find, and fix the spec.
8. Read `dist/<package>/<rubric>.cells.md` against the rubric's structural lines, one line at a time, `whole.1` to the last. A line that does not hold is a fix to the spec and a rebuild, not a note. For a workbook, open the result in LibreOffice or Excel, or convert it headless to pdf, and read the computed tabs the rubric names, since the filler keeps formulas but computes none.
9. Report to the owner as a numbered list: the structural lines that hold, the ones that do not and why, the values a computed tab shows, and the file's path. Name the upload filename the rubric demands.
10. When the owner sends the file, copy it as sent to the workspace package's `assets/submissions/<date>-<rubric>.<ext>`, and file the reader's feedback beside it when it arrives.
