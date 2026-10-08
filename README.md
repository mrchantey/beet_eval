# beet_eval

Evaluates a subject against packages of evals and rubrics: mechanical checks, grades with verbatim evidence, results every rubric reads, and the next unit of work. Prose documents, markdown or Word, are the first subject and Office forms the renderer, with four agent roles writing, grading, building and coaching through its routes.

A downstream of [beet](https://github.com/mrchantey/beet), depended on by path: this repo's own beet binary, `beet-eval` (`just cli`), is the stock runner plus `EvalPlugin`, serving `main.bsx`.

```sh
just cli --help
just test
```

This page is the one source for the words. The law lives in the module docs, which cite it: `eval` for evals, rubrics, grades and results, `docs` for the documents, each a markdown or Word file read through beet's media parse, and their checks, `form` for building a form, `coach` for the claims register and the coach's actions, `routes` for the verbs.

## Glossary

| Word | Type | Meaning |
|---|---|---|
| subject | | The thing under evaluation. Documents under `docs/` are the first kind. |
| eval | `Eval`, `EvalId` | One statement about the subject, true of a strong subject and false of a missing one, defined once as a row keyed `namespace.slug` and never renamed. |
| level | `EvalLevel`, `Levels` | One of the four steps of the scale, 0 missing, 1 stated, 2 sound, 3 strong. An eval's `Levels` say how it is decided: the scale as written (`Generic`), met or not (`Binary`), or four pinned lines (`Custom`). |
| check | `CheckRef`, `CheckVerdict`, `CheckOutcome` | A binary eval decided by a route under `check/` with the params its `CheckRef` carries, rather than by a grader. The route answers a `CheckVerdict`; a run keeps its pass and detail as a `CheckOutcome`. A new check kind is a new route. |
| judged eval | | An eval with no check, graded by a grader. |
| source tag | `SourceTag` | Where a statement comes from, written in brackets, ie `[ref:sba]`, and defined in the source table of the package that owns the source. |
| address | `Address` | A place in the subject, `document` or `document#section`, the section half being the heading's slug. An eval's anchor, a block's home and a claim's block are addresses. |
| rubric | `Rubric`, `RubricSection`, `RubricRef` | A profile for one reader: the form's headings, the evals each cites at a level, and the structural lines of the form. Named across a workspace as `<package>/<rubric>`. |
| citation | `Citation` | An eval cited at the level a reader demands, 1 to 3. Read off the results, never off a rendered form. |
| structural line | `StructuralLine` | A checkable sentence about a rendered form, addressed `label.n`. Checked on the form, never on `docs/`. |
| grade | `Grade` | One grader's level for one judged eval with verbatim evidence from its anchor, written only through `eval/grade`. |
| results | `Results`, `GradeSet`, `RubricResult`, `CitationResult`, `CitationStatus` | One run: every check decided, the grades merged, and every rubric's citations read as met, failing or awaiting. |
| next step | `NextStep`, `NextVerb`, `DocumentRow`, `RubricRow` | The next unit of work, why, and the tables the choice was made from. |
| package | `PackageManifest`, `PackageKind`, `LoadedPackage`, `PackageTable` | A directory in a store with a manifest. A **document** package says what the documents are, a **reader** package holds what one outside reader needs, a **workspace** package is the subject's own. |
| workspace | `Workspace`, `PackageSource`, `LoadedWorkspace`, `PackagedEval` | A store holding `docs/`, `results/` and a manifest naming its packages, read for a verb with every package and every eval of all of them. |
| outline | `Outline`, `DocumentSpec`, `SectionSpec`, `BlockSpec`, `HistoryEntry` | A document package's list of the documents, their sections and the data blocks they carry. It generates the evals that restate it. |
| document | `DocumentSet`, `DocumentFile`, `DocumentSection`, `ProseQuery` | One of the subject's documents, a markdown or Word file under `docs/` or a directory promoted from one, parsed by beet's media parse into a tree a `DocumentSet` holds: frontmatter, title, tagline, summary and `##` sections, each read by a `ProseQuery` traversal. Its frontmatter is beet's `PageMeta`, carrying `created`, `updated` and `authors`, a Word file's from its core properties. |
| data block | `DataBlock`, `BlockFormat`, `Column`, `ColumnKind` | A fenced `csv` or `json` block named in its info string, defined once across the documents. |
| ask | `Ask`, `AskKind` | An open question to the owner, inline as `TODO(ask ...)`: a **fact** a source could answer, offered with a default, or a **decision** only the owner can make, challenged by the coach instead. |
| check kind | `CheckRoutes`, `DocumentCheck`, `FrontmatterCheck`, `H1Check`, `TaglineCheck`, `SummaryCheck`, `AgreesCheck`, `AsksCheck`, `SectionsCheck`, `BlockCheck`, and the params of each, `DocumentCheckParams`, `FrontmatterCheckParams`, `H1CheckParams`, `TaglineCheckParams`, `SummaryCheckParams`, `AgreesCheckParams`, `TitlePart`, `AsksCheckParams`, `SectionsCheckParams`, `BlockCheckParams` | The document check kinds, one route each under `check/`, mounted by `<CheckRoutes/>`. |
| render spec | `RenderSpec` | How a reader package renders one form: the reader's own blank form, a Word file, a workbook or any document with tables, the output's name, and the form's quirks. |
| fill spec | `FillSpec`, `FillOp`, `CellRef`, `CellsReport` | What a builder writes for one form: the operations that fill the blank form, by cell, each one of beet's edits, and the cells of the result it reads back. |
| claim | `Claim`, `Stakes`, `ClaimStatus` | A row of the workspace package's claims register: something the plan rests on that might be false, with its evidence 0 to 3, its stakes, its test and its status. |
| action | `CoachAction` | One of the coach's moves, a row of a document package's actions table: the ask, when to stop pushing, the red flags, and a bad and a good exchange. |
| verb | `EvalRoutes`, `EvalCheck`, `EvalResults`, `EvalNext`, `EvalBlocks`, `EvalWorksheet`, `EvalProject`, `EvalNew`, `EvalGrade`, `EvalPut`, `EvalDrop`, `EvalCells`, `EvalBuild` | One route of `beet-eval` under `eval/`, mounted by `<EvalRoutes/>`. |
| report | `ResultsReport`, `NextReport`, `WorksheetReport`, `WorksheetDocument`, `WorksheetSection`, `BlocksReport`, `BlockDump`, `BlockEntry`, `ProjectionReport`, `Projection`, `ProjectedHeading`, `ProjectedCitation`, `CheckReport`, `CheckSummary`, `ScaffoldDocument`, `ScaffoldSection`, `ScaffoldBlock` | A verb's answer as a scene, a `#[template]` of HTML nodes beet renders as the request accepts, beside the data it shows, which a serde `Accept` answers instead; a file a verb writes is the scene's markdown. |
| clerk, grader, builder, coach | | The four agent roles, below. |

## Layout

A package and a workspace are stores, addressed by uri and never assumed to be on disk. Every structured thing in them is a row of a table or a typed document; prose for people, the documents under evaluation and the READMEs, stays markdown.

```
<package>/
  package.json          PackageManifest
  README.md             what the package is and provides, for a person
  outline.json          Outline, a document package's
  evals/<id>            Eval rows
  rubrics/<id>          Rubric rows
  render/<rubric>       RenderSpec rows, a reader package's
  actions/<id>          CoachAction rows, a document package's
  claims/<id>           Claim rows, a workspace package's
  assets/               the files the package provides or was built from

<workspace>/
  workspace.json        Workspace
  docs/                 the documents under evaluation, markdown or Word
  results/grades/<id>   Grade rows, keyed on the eval
  results/summary.json  Results, the last run
  results/projections/  the briefs eval/project writes
  dist/<package>/       builds: the filled form, its <rubric>.fill.json FillSpec, its <rubric>.cells.md
  packages/<name>/      the workspace's own package
```

A table is one blob per row at `<table>/<key>`, the key being the row's id verbatim with no extension, pretty-printed JSON, so a tool call writing one row touches one object and a change diffs by field. The JSON is serde's default form of the Rust type: fields by name in sorted order, enum variants by their Rust name (`"Generic"`, `{"Binary": {...}}`), a newtype as its inner value, an absent option as `null`. The schema is derived from the same type by `ValueSchema::of`, so a form can be generated for each and the stored form validates against it. Every example below is tested to deserialize into its type, validate against its schema and serialize back unchanged.

### A package

`package.json`, the manifest. A source key is a tag, or a family of tags with its varying parts in angle brackets; a reference to the literature is a `ref:<slug>` entry.

```json PackageManifest
{
  "builds_on": [],
  "kind": "Document",
  "name": "acme_biz",
  "outline": "outline.json",
  "sources": {
    "acme:outline": "This package's own outline.json.",
    "mc:<topic>/<slug>": "A masterclass recording, by topic and slug.",
    "ref:sba": "US Small Business Administration, Write your business plan: the traditional outline the documents follow."
  }
}
```

`outline.json`, a document package's outline. It generates, in the `structure` namespace, `<document>-present` and `<document>-sections` for each document, `<block>-block` for each block, and `frontmatter-complete` and `summaries-present` over the set.

```json Outline
{
  "blocks": [
    {
      "columns": [
        { "kind": "Text", "name": "line" },
        { "kind": "Num", "name": "price_ex_gst" }
      ],
      "lives_in": "product#pricing",
      "name": "price-list"
    }
  ],
  "documents": [
    {
      "name": "product",
      "sections": [
        { "heading": "Idea", "holds": "Where it came from, why it will work, and the alternatives it was preferred to." },
        { "heading": "Pricing", "holds": "The strategy and the unit economics of each line; the price-list block." }
      ],
      "summary": "What the business sells and how it delivers it."
    }
  ],
  "history": [
    { "change": "First draft.", "date": "2026-10-02" }
  ],
  "source": "acme:outline"
}
```

`evals/product.alternatives-considered`, a judged eval with its own level lines.

```json Eval
{
  "anchor": "product#idea",
  "id": "product.alternatives-considered",
  "levels": {
    "Custom": {
      "l0": "No alternative is named.",
      "l1": "Alternatives are named without a reason for passing them over.",
      "l2": "Each alternative carries why the chosen idea beat it.",
      "l3": "The alternatives were weighed on the same grounds as the chosen idea, and the comparison is written down."
    }
  },
  "note": null,
  "sources": ["inferred"],
  "statement": "Other ideas were considered and the chosen idea is preferred to them for a reason."
}
```

`evals/structure.index-tagline-agreed`, a checked eval the package writes itself.

```json Eval
{
  "anchor": null,
  "id": "structure.index-tagline-agreed",
  "levels": {
    "Binary": {
      "check": {
        "params": { "document": "index", "part": "Tagline", "section": "brand#name-and-tagline" },
        "route": "agrees"
      }
    }
  },
  "note": null,
  "sources": ["acme:outline"],
  "statement": "The tagline beneath the index's title appears under the brand document's name and tagline section, which owns it."
}
```

`rubrics/01-business-plan`, a reader package's rubric. A section's `sections` nest the form's subheadings.

```json Rubric
{
  "id": "01-business-plan",
  "intro": "The plan an assessor marks before coaching starts.",
  "reader": "an assessor marking the form satisfactory",
  "sections": [
    {
      "agrees_with": [],
      "beyond_minimum": null,
      "citations": [],
      "example": null,
      "label": "whole",
      "prose": "The form as uploaded.",
      "sections": [],
      "sent_back_when": ["A red instruction sentence is left in. [form]"],
      "structural": [
        { "sources": ["form"], "text": "Every red instruction sentence is deleted." }
      ],
      "title": "Whole document"
    },
    {
      "agrees_with": ["the canvas's Value Proposition block"],
      "beyond_minimum": null,
      "citations": [
        { "eval": "product.alternatives-considered", "level": 2 }
      ],
      "example": "The demonstration names one rejected idea. [demo]",
      "label": "1.2",
      "prose": "What the business is and why a customer chooses it.",
      "sections": [],
      "sent_back_when": [],
      "structural": [],
      "title": "1.2 Business Description"
    }
  ],
  "sources": ["the blank form", "the trainer's guide"]
}
```

`render/01-business-plan`, a reader package's render spec.

```json RenderSpec
{
  "form": "assets/forms/01-business-plan.docx",
  "notes": "Tick the applicable boxes with Check by label, delete every red instruction sentence, and write in the third person.",
  "output": "01-business-plan.docx",
  "rubric": "01-business-plan"
}
```

`actions/status-quo`, one of a document package's coach actions.

```json CoachAction
{
  "ask": "What do they do about this today, and what does it cost them?",
  "bad": "Owner: nobody does anything about it. Coach: great, the field is open.",
  "good": "Owner: they keep a spreadsheet. Coach: who keeps it, and how many hours a week does it take?",
  "id": "status-quo",
  "push_until": "The owner names the current workaround and what it costs.",
  "red_flags": ["Nobody does anything about it.", "They would if they knew about us."],
  "serves": ["market#competitors"],
  "sources": ["ref:fitzpatrick-2013"],
  "statement": "The customer's current workaround, and its cost, is known."
}
```

`claims/buyers-pay-monthly`, a row of a workspace package's claims register.

```json Claim
{
  "block": "finance#sales-projection",
  "claim": "Market stall holders will pay a monthly fee rather than a one-off price.",
  "evidence": 1,
  "id": "buyers-pay-monthly",
  "sources": ["interview 2026-10-01"],
  "stakes": "Fatal",
  "status": "Open",
  "test": "Ask three stall holders to commit to a first month at the listed price."
}
```

### A workspace

`workspace.json`. Its own package sits in the workspace store; any other is a store uri, a relative `fs:` path resolving against the directory the workspace runs from.

```json Workspace
{
  "dist": "dist",
  "docs": "docs",
  "packages": [
    { "Local": "packages/acme" },
    { "Store": { "Fs": { "path_prefix": "../beet_eval/packages/acme_biz" } } }
  ],
  "results": "results"
}
```

`results/grades/product.alternatives-considered`, a grade.

```json Grade
{
  "anchor": "product#idea",
  "by": "a grader",
  "date": "2026-10-04",
  "eval": "product.alternatives-considered",
  "evidence": "we weighed a mobile service against the stall and chose the stall for its foot traffic",
  "level": 2
}
```

`results/summary.json`, the last run, rendered for a person by `eval/results`.

```json Results
{
  "checks": [
    { "detail": "\"TODO(ask): the tagline\" is not under brand#name-and-tagline", "eval": "structure.index-tagline-agreed", "pass": false }
  ],
  "grades": {
    "by": ["a grader"],
    "date": "2026-10-04",
    "problems": [],
    "rows": [
      {
        "anchor": "product#idea",
        "by": "a grader",
        "date": "2026-10-04",
        "eval": "product.alternatives-considered",
        "evidence": "we weighed a mobile service against the stall and chose the stall for its foot traffic",
        "level": 2
      }
    ]
  },
  "ran_at": "2026-10-04",
  "rubrics": [
    {
      "citations": [
        { "eval": "product.alternatives-considered", "level": 3, "status": { "Failing": { "got": 2 } } },
        { "eval": "structure.index-tagline-agreed", "level": 2, "status": { "Failing": { "got": 0 } } }
      ],
      "id": "acme_biz/owner"
    }
  ]
}
```

What `eval/next` answers.

```json NextStep
{
  "documents": [
    {
      "asks": 3,
      "document": "brand",
      "graded": 0,
      "judged": 9,
      "owner_gap": 21,
      "present": true,
      "reader_gap": 4,
      "shape_failures": ["structure.index-tagline-agreed"],
      "stale": false,
      "updated": "2026-10-04"
    }
  ],
  "rubrics": [
    { "awaiting": 0, "failing": 2, "met": 0, "rubric": "acme_biz/owner" }
  ],
  "targets": ["brand"],
  "verb": "Write",
  "why": "docs/brand fails 1 shape check: structure.index-tagline-agreed. Shape before substance."
}
```

`dist/acme_course/01-business-plan.fill.json`, a builder's fill spec. A Word cell is `t<table>r<row>c<cell>`, a workbook cell `<sheet>!<A1>`.

```json FillSpec
{
  "ops": [
    { "Set": { "cell": "t1r2c2", "text": "Acme Stalls" } },
    { "Append": { "cell": "t3r1c1", "text": "Acme Stalls rents fitted market stalls by the month." } },
    { "Check": { "label": "Primary research" } },
    { "Delete": { "text": "Please delete this sentence once completed." } },
    { "Replace": { "new": "Acme Stalls", "old": "(Insert business name)" } },
    { "Set": { "cell": "Start Here!D3", "text": "Acme Stalls" } }
  ]
}
```

## The verbs

Each is a route of `beet-eval`, its flags on a params type so `--help` documents them, resolving the workspace through the store above it. A workspace runs them from its own entry, `eval.bsx` at its root, mounting `<EvalRoutes/>` under `eval`, `<CheckRoutes/>` under `check`, beet's `<BlobCells/>` under `blob` and beet's `<BlobView/>`, so the entry's directory, the repo store, is the workspace store; `tests/fixtures/acme/eval.bsx` is one:

```sh
beet-eval --main=eval.bsx eval/next
```

A report answers a scene rendered as the request accepts: ANSI in a terminal, markdown when piped, ie to an agent or a file, HTML in a browser, or with `--accept=application/json` the stored form of what it computed. A verb that finds something wrong answers it and exits 1.

- `eval/check [--unused]`: every table's rows, global ids, the prose rules, each check's params against its route's, citations and anchors against the outline; `--unused` lists the evals no rubric cites.
- `eval/results`: runs every check, merges the grades, reads every rubric, writes `results/summary.json`, and answers the run, its line first.
- `eval/next`: the next step, with the document and rubric tables behind it.
- `eval/blocks [<name>]`: lists the data blocks, every block in full as JSON, or prints one's body as text.
- `eval/worksheet [<document>]`: every judged eval anchored in each document, with its statement and level lines.
- `eval/project <package>/<rubric>`: the brief for one form, each heading's evals, the text at their anchors and its structural lines, answered and written to `results/projections/`.
- `eval/new <name> <author>`: scaffolds `docs/` from the outline, every body an ask; refuses a non-empty `docs/`.
- `eval/grade --eval --level --by [--anchor] [--evidence]`: writes one grade, refused unless the level is one the eval admits, at most 1 where an ask is open, and the evidence is verbatim from its anchor.
- `eval/put --package --table (--row | --from)`: writes one row of a package's table, given inline or drafted into the workspace store and named by its path, refused unless it keeps the law `eval/check` holds every row to.
- `eval/drop --package --table --key`: removes one row of a package's table, refused while a rubric cites the eval or a render spec builds the rubric.
- `eval/cells <package>/<rubric>`: the cells of a reader's blank form, the map a fill spec is written against.
- `eval/build <package>/<rubric>`: parses the reader's blank form, applies the fill spec, renders the result into `dist/` in the form's own format, and writes its cells beside it.
- `check/<kind>`: one document check kind, called by `eval/results` for every checked eval and directly for one, answering a `CheckVerdict`.
- `blob/cells <path>`: beet's cells of any file in the store with tables, a Word file, a workbook or a markdown form, the map a fill spec is written against.
- `view <path> [--accept=text/markdown]`: beet's view of any file in the store, parsed as its media type and rendered as the request accepts, so a Word file, a workbook or a slide deck reads as markdown: headings, lists, GFM tables headed `<!-- t<n> -->` with the cells' numbers, checkboxes as `[ ]` and `[x]`, and what a form signals by look kept as `<mark>` and `<span style>`.

## The roles

The loop is a tree: `eval/next` names the unit of work and dispatches to the role whose turn it is, one unit per turn, until it answers done. Each role is an agent actor carrying the routes of its trade as tools and its procedure as its seed post, and every role can be spoken to, turn by turn in text or as a realtime voice session.

- **Clerk**: writes a document from its sources and the owner's answers, drafting what the sources allow and asking what only the owner knows.
- **Grader**: grades the documents against the worksheet through `eval/grade`, reading a store it cannot write.
- **Builder**: fills a reader's form, writing the fill spec from the brief and reading the built form's cells against the structural lines.
- **Coach**: runs a sitting against the claims register, one question per turn by the actions table, ending with one assignment written as the picked claim's test.
