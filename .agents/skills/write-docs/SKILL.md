---
name: write-docs
description: >
  Write or revise the documents under a beet_eval workspace's docs/ with the owner, one
  document at a time: scaffold with `just eval new`, then the interview loop, draft what the
  sources allow, ask only what the owner alone knows with a default stated, write the answers
  in, file the sitting. Use when asked to write, draft, fill, scaffold or improve the
  documentation, or when `just eval results` shows a document failing.
---

# write-docs

The owner supplies the claims, the specifics, the decisions and the taste; the agent supplies structure, completeness, consistency and the next question. Read three things before the first sitting: beet_eval's `src/markdown/mod.rs` (`~/me/beet_eval`), the shape of a document, its asks and its writing rules; the document package's `outline.json` and `README.md`, what each document holds and why; and `just eval worksheet <document>`, the bar, every eval anchored there with its level lines.

## Sources

The workspace package's `assets/`: the founding notes, the interview files from earlier sittings, registrations and quotes as they arrive. `just eval results --format=md` and the `results/grades/` table for where the documents stand. A reader package's assets only for what its form will demand, never as a source of facts about the subject.

## Scaffold, once

1. When `docs/` is empty, `just eval new "<Business name>" "<Author>"`. It writes every document of the outline with its sections, an empty data block under each section that owns one, and an ask in every body, so `just eval results` fails on the asks until the loop has been through.

## Order

2. The index first, as hypotheses: tagline, pitch, mission, canvas, goals. It is where the owner's thinking is sharpest and it constrains everything below. Then the documents in the outline's order. Then the index again, revised against the bodies.

## One sitting

3. Pick one document, or one section of a large one. Draft what the sources support, by the writing rules: first person plural or the subject's name, present tense, one idea per sentence, every number in a data block, a `Sources:` line where the facts came from somewhere.
4. List the asks: the fewest concrete questions the sources cannot answer, in the order a stranger would ask them, each with a proposed default so the owner can say yes or correct it. Put them in the response as a numbered list with lettered options, a) the default. No more than about eight in one sitting.
5. When the answers arrive, write them in. Keep the owner's phrases where they are good; cut a sentence that could be said of any subject; put every figure in its block; leave an ask inline for anything still unknown, `TODO(ask fact: ...)` where a source could answer it and `TODO(ask decision: ...)` where only the owner can. Move `updated` in the frontmatter.
6. File the sitting under the workspace package's `assets/interviews/<date>-<document>.md`: the questions asked, the answers as given with the owner's words kept where they were used, and what was written where. Every fact in `docs/` traces to one of these or to an asset.
7. `just eval results`. The document's failing evals are the next sitting's agenda.

## Rules of the loop

8. When the owner edits a draft directly, read the diff before the next sitting; the edits are the voice to match, and the last few diffs are the style examples for what comes next.
9. A section's prose stays under about two hundred words unless it is a register; a paragraph carries a name, a number, a date or a place; a claim carries its evidence or its source, and a number without a source says it is an estimate.
10. Never invent a source, a quote from a customer, a price or a figure. An unknown is an ask, not a guess dressed as a fact.
11. When a document settles, hand it to `grade-docs`.
