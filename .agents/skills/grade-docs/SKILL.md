---
name: grade-docs
description: >
  Grade the documents under a beet_eval workspace's docs/ against every judged eval, one
  grade at a time through `just eval grade`, which writes the results' grades table that
  `just eval results` merges. Use when asked to grade, score, test or assess the
  documentation, when the results report judged evals awaiting, when docs/ has changed
  since the last grade, or before a build.
---

# grade-docs

The grader is you. beet_eval's `src/eval/mod.rs` (`~/me/beet_eval`) is the law: the four-level scale, what a judged eval is, and what a grade must carry, its sections The scale and Grades and results above all; read them before the first grade. Nothing here edits `docs/`; a grader who wants to fix a document stops and says so.

## Before grading

1. `just eval check` must be clean and `just eval results` run once, so the shape failures are known. Grade regardless of them: a missing section is a 0 on everything anchored there, which is information.
2. `just eval worksheet` is the worksheet: every judged eval anchored in each document, in the outline's order, with its statement and its level lines. `just eval worksheet <document>` for one document.

## Grading one eval

3. Open the document, `docs/<name>.md` or `docs/<name>/index.md` and its children. For each eval on the worksheet read the section its anchor names, and the whole document when the anchor is the document itself.
4. Assign the level by the eval's own level lines when it has them, by the scale when it is generic, and 0 or 2 when it is binary. Between two levels, the lower. A section with an ask open caps every eval anchored there at 1. Grade only what is on the page: nothing from the interview files, nothing from memory, nothing from what the owner meant.
5. Write the grade, one eval per call:

```sh
just eval grade --eval=product.origin --level=2 --anchor='product#idea' \
  --evidence='began as a tool we built for our own weekend market stall' --by='<model or person>'
```

The anchor is where the evidence sits, defaulting to the eval's own; the evidence is a verbatim quote of at most twenty-five words from it, left out only at 0 when nothing is written there; `--by` is the grader of record, the same name for the whole pass. The verb refuses a checked or unknown eval, a level the eval does not admit, a level above 1 where an ask is open, and a quote not found at the anchor: read the refusal, fix the grade, write it again. A grade replaces any earlier grade of the same eval.

6. Every judged eval gets a grade, 0 included. When a pass holds many documents, fan out one sub-agent per document with the same rules and the same `--by`; each writes its own rows, so nothing needs merging.

## After grading

7. `just eval results`, then `just eval results --format=md` for the per-rubric failing lists. Report to the owner by rubric: the ids that fail, the level needed and the level reached, in a numbered list, not the table.

## Regrading

8. When a document's `updated` is later than its grades, `just eval next` shows it stale: regrade every eval anchored in that document; the rest of the grades stand. A grade of a document that has since changed is stale, and the results say so, so do not leave it.
