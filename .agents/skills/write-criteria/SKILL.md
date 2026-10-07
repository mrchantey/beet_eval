---
name: write-criteria
description: >
  Add, change or extract evals in a document package's evals table, the
  audience-independent statements every document is graded against, writing each row
  through `just eval put`. Use when a rubric needs an eval the package lacks, when a source
  (a template, a guide, a recording, a book) yields new evals, when two evals turn out to be
  one, or before changing any row of a package's evals table.
---

# write-criteria

beet_eval's `src/eval/mod.rs` (`~/me/beet_eval`) is the law: what an eval is, judged or checked, the id rule, the four-level scale, the fields and what is not an eval; its `README.md` gives the stored shape of a row. The document package's `README.md` is the scope of each namespace and its `outline.json` what each document holds. Read them before touching the table. Every row is written through `just eval put`, which refuses a row that breaks the law, and the whole set is held to it after every change:

```sh
just eval check --unused
just eval results
```

## Adding an eval

1. Search first. The table is one JSON file per row, `<package>/evals/<id>`: list the ids in the namespace and read the statements of anything nearby, `grep -l '<word>' <package>/evals/*` for the rest. Most new evals are an existing one seen from another template, and the fix is a citation in the rubric, not a new row. If an existing statement is too narrow, widen it rather than adding a sibling, provided every rubric citing it still means the same thing.
2. Choose the namespace by the scope table and the slug by what is judged, not where it came from: `competitors-named`, never `bp-2-3-competitors`. Set the anchor, the `document#section` where a strong document satisfies it.
3. Write the statement so that it is true of a strong document and false of a missing one, in words any subject could be held to. A count that a form demands stays out; a count intrinsic to the idea ("both years") stays in.
4. Choose the levels. `{"Binary": {"check": null}}` when the thing is either done or not and a grader decides it. `"Generic"` when the scale's own words already say what 1, 2 and 3 look like for this statement. `{"Custom": {..}}` when they do not, pinning all four lines, each strictly harder than the last, each a sentence a grader can test. A check, `{"Binary": {"check": {"route": "<kind>", "params": {..}}}}`, only when a route under `check/` decides it; `just eval --help` lists the kinds and their params.
5. Tag the sources, most authoritative first, each a key of the owning package's manifest `sources`, a book or a standard as a `ref:<slug>` entry there. A new tag gets its entry in the manifest before a row uses it. Add a `note` for any contradiction between sources or accepted not-applicable answer.
6. Write the row, inline for a short one or drafted into the workspace store for a long one:

```sh
just eval put --package=<package> --table=evals --row='{"anchor": "market#competitors", "id": "market.competitors-named", ..}'
just eval put --package=<package> --table=evals --from=.agents/tmp/criteria/market.competitors-named.json
```

7. `just eval check --unused`. Then cite the new eval from the rubric that needed it.

## Changing one

8. An id is permanent. Read the row, edit it and put it back under the same id: wording can be tightened when every citing rubric still means the same thing. A change of meaning is a new eval, every rubric repointed at it, then the old row removed with `just eval drop --package=<package> --table=evals --key=<old id>`, which refuses while any rubric still cites it and names them. A move between namespaces when the outline's documents change keeps the slug, rewrites every citation in the same change and is recorded in the package README's History. Merging two evals keeps the better slug and repoints the other's citations.

## Extracting from a source

The procedure for any new source, whether a template, a guide, a recording or a book:

9. **Classify.** Walk the source requirement by requirement and mark each structural (a shape or count the form demands), an eval (a quality any document of the kind has), or an agreement (one document must match another). Only the middle kind enters the table.
10. **Draft candidates** as rows, into scratch under `.agents/tmp/criteria/<source>/<namespace>/<slug>.json`, keeping every source tag the requirement carried. Where several requirements are one idea at different bars, draft one eval with custom levels.
11. **Merge** the candidates into the table: read every candidate against the existing rows, fold duplicates, keep the more general statement, put each survivor with `--from`, and write an alias list `.agents/tmp/criteria/aliases.md` from every candidate id to its final id, so the rubrics extracted alongside can be rewritten mechanically.
12. **Rewrite the rubrics** as profiles, by `write-rubric`, citing the merged ids, then `just eval check --unused`.
