---
name: iterate
description: >
  Run one turn of a beet_eval workspace's loop without being told where to start: let
  `just eval next` name the document, grading pass or form that needs the most work, do that
  one thing by the matching skill, and leave the results current. Use when asked to iterate,
  continue, keep going, do the next thing, or work on the docs with no other instruction;
  when the owner names a document, a verb or a form, start there instead; and when the
  owner's message answers the asks of the last sitting.
---

# iterate

One turn is one unit of work: a sitting on one document, one grading pass, or one build. It ends when the owner's input is needed or the unit is done, with the results current and the next step named. Where to start is `eval/next`'s judgement, computed from the last checks, grades, open asks and edit dates, not yours, unless the owner overrides it.

The verbs are beet_eval's, `~/me/beet_eval`, whose `README.md` fixes the words and whose module docs carry the law. A workspace runs them from its root as `just eval <verb>`, a bare verb being `eval/<verb>`; `just eval --help` lists every verb with its flags. A verb's report answers markdown when its output is piped, and ANSI to a terminal, so where a harness runs commands in one, add `--accept=text/markdown`; `--accept=application/json` answers its stored form. The skills named below sit beside this one.

## Orient

1. `just eval check`. If it fails, stop: a package is broken, which `write-criteria` or `write-rubric` fixes, and say so.
2. `just eval results`, then `just eval next`. The heading is the recommendation; the tables beneath are why.

## Choose

3. In this order: the owner's message names a target, a document, "grade", or "build <package>/<rubric>", then that; the owner's message answers the asks of the last sitting, then write them in, step 5 from "when the answers arrive"; otherwise the heading of `just eval next`.

## Act

4. **scaffold**: `just eval new "<Business name>" "<Author>"`, the name and author from the workspace's `AGENTS.md` or `README.md`, asked for once if neither says. Then a sitting on the index in the same turn.
5. **write <document>**: one sitting by `write-docs`, with `just eval worksheet <document>` for the bar. Mend first any shape failure `just eval next` listed, a missing section, a misplaced block, a lost tagline, since everything anchored there is blocked by it. Draft what the sources support. Put the asks to the owner, fewest first, each with a default. When the answers arrive, write them in, file the sitting under the workspace package's `assets/interviews/`, move `updated`, then `just eval results`.
6. **grade <documents>**: `grade-docs` for the documents named, every eval anchored in them, then `just eval results`.
7. **build <package>/<rubric>**: `build-form`, then `just eval results`.
8. **done**: say so, and what would raise the bar further: the owner evals held at 2 by choice, the forms built but not yet sent.

## Report

9. A numbered list: what `just eval next` said and why, in a line; what was done, with the files touched; the asks waiting on the owner, numbered, each with its default; what `just eval next` says now. Not the tables: `just eval results --accept=text/markdown` answers them for anyone who wants them.

## Rules

10. One unit per turn. A sitting that ends in asks does not grade its own draft; the grade comes after the answers are in.
11. Shape before substance: a document with a failing structure check is mended before prose is written into it.
12. Nothing goes into `docs/` that the owner did not say or a source does not carry; an unknown is an ask, never a guess.
13. The owner's override wins over `just eval next` and is said back in the report.
