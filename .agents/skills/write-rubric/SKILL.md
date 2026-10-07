---
name: write-rubric
description: >
  Write or extend a rubric: a profile over the evals for one reader and purpose, naming the
  evals it cites, the level each must reach, and the structural lines of that reader's form,
  one row of a package's rubrics table written through `just eval put`. Use when asked to
  write, update or review a rubric, when a new form or document type appears, or when a
  source a rubric cites changes.
---

# write-rubric

A rubric is a suite: it cites evals from the document package (the tests) at the level one reader demands, and adds the structural lines only that reader's form has. It never restates an eval. beet_eval's `src/eval/mod.rs` (`~/me/beet_eval`), its sections Rubrics and What is not an eval, is the law; its `README.md` gives the stored shape of a `Rubric` row, which a section's `sections` nest. Two families share the shape: a reader package's rubrics, one per form, whose bar is what that reader accepts, and a document package's owner rubric, whose bar is the best documentation a subject can hold itself to. A rubric's citations are read off the one set of results the documents' grades produce, never off a rendered form, so a form asking for a summary of something still holds the subject to the whole eval at the level cited; its structural lines are checked on the rendered form, since they are about the form.

## The shape

The row is `rubrics/<id>`: an `id`, the `reader` and an `intro` saying what the form is, who reads it and on what basis, where it goes and what feeds it; `sources`, every file the rubric was built from, most authoritative first; and `sections` following the form's own headings. Each section carries:

- `title`, the form's heading exactly, even an odd one, so a rubric address is also a form address, and `label`, the heading's own number without a trailing dot: `2.0` for a numbered top-level heading that carries its own lists above its subsections, a lowercase word for an unnumbered heading, `whole` for "Whole document". A workbook uses its tab names, with the scheme stated in the intro.
- `prose`, what the section asks in the form's terms and what the reader looks for.
- `citations`, each `{"eval": "<id>", "level": <1 to 3>}`, 2 only for a binary eval. The prose says why; the eval says what.
- `structural`, each `{"text": "<one checkable sentence>", "sources": [..]}`: a cell filled, a count of rows, a sentence deleted, a mark made bold. Its address is `label.n`, so a result can cite it: append to a list, never insert.
- `sent_back_when`, `agrees_with`, `example` and `beyond_minimum`, only when a source gives them something to say; a `sent_back_when` entry keeps its source tags inline.

Which list a requirement belongs in: a shape or count the form demands is structural; a quality any document of the kind would have is a citation, and if the package lacks the eval, add it by `write-criteria` rather than writing it here. A requirement that one document match another is neither: it goes in `agrees_with`.

## Source tags

A tag is defined by the manifest `sources` of the package that owns the source: a reader package's for everything that reader published, its order of authority the order a rubric's sources follow; `ref:<slug>` for the literature; `inferred` is a legal source and an honest one. A new source gets its entry in the owning manifest before a rubric uses it.

## Building one

1. Read the blank form in full, `just eval view <path to the form> --accept=text/markdown`. Every fill-in cell, checkbox list and red sentence is a candidate requirement; keep the form's numbering and labels.
2. Read the reader's guidance for the form: its guide, its recordings, its feedback on earlier submissions. Bold in a guide is a minimum.
3. Read the reader's demonstration, if it has one, and note where it falls short of a stated minimum.
4. For every substantive requirement, find the eval in the document package and cite it at the level this reader demands; add one by `write-criteria` only when nothing fits. Everything about the form goes in `structural`.
5. Draft the row in the workspace store, `.agents/tmp/rubrics/<package>-<id>.json`, and put it: `just eval put --package=<package> --table=rubrics --from=.agents/tmp/rubrics/<package>-<id>.json`. The verb refuses an unknown eval or a level it cannot reach.
6. `just eval check --unused`, then read the row back against the form once: every heading present, every fill-in covered by a citation or a structural line. Update the package README's index of its rubrics, if it keeps one.

A rubric is honest before it is complete: a requirement nobody stated is tagged inferred, a number nobody gave is not invented, and a contradiction between sources is written down rather than resolved silently.
