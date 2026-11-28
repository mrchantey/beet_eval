# beet_eval

Evaluates a subject against suites of evals and rubrics: mechanical checks, grades with verbatim evidence, results every rubric reads, and the next unit of work. Prose documents, markdown or Word, are the first subject and Office forms the renderer, with four agent roles writing, grading, building and coaching through its routes.

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
| source tag | `SourceTag` | Where a statement comes from, written in brackets, ie `[ref:sba]`, and defined in the source table of the suite that owns the source. |
| address | `Address` | A place in the subject, `document` or `document#section`, the section half being the heading's slug. An eval's anchor, a block's home and a claim's block are addresses. |
| rubric | `Rubric`, `RubricSection`, `RubricRef` | A profile for one reader: the form's headings, the evals each cites at a level, and the structural lines of the form. Named across a workspace as `<suite>/<rubric>`. |
| citation | `Citation` | An eval cited at the level a reader demands, 1 to 3. Read off the results, never off a rendered form. |
| structural line | `StructuralLine` | A checkable sentence about a rendered form, addressed `label.n`. Checked on the form, never on `docs/`. |
| grade | `Grade` | One grader's level for one judged eval with verbatim evidence from its anchor, written only through `eval/grade`. |
| results | `Results`, `GradeSet`, `RubricResult`, `CitationResult`, `CitationStatus` | One run: every check decided, the grades merged, and every rubric's citations read as met, failing or awaiting. |
| next step | `NextStep`, `NextVerb`, `DocumentRow`, `RubricRow` | The next unit of work, why, and the tables the choice was made from. |
| suite | `SuiteManifest`, `SuiteKind`, `LoadedSuite`, `SuiteTable` | A directory in a store with a manifest. A **document** suite says what the documents are, a **reader** suite holds what one outside reader needs, a **workspace** suite is the subject's own. |
| workspace | `Workspace`, `SuiteSource`, `LoadedWorkspace`, `SuiteEval` | A store holding `docs/`, `results/` and a manifest naming its suites, read for a verb with every suite and every eval of all of them. |
| outline | `Outline`, `DocumentSpec`, `SectionSpec`, `BlockSpec`, `HistoryEntry` | A document suite's list of the documents, their sections and the data blocks they carry. It generates the evals that restate it. |
| document | `DocumentSet`, `DocumentFile`, `DocumentSection`, `ProseQuery` | One of the subject's documents, a markdown or Word file under `docs/` or a directory promoted from one, parsed by beet's media parse into a tree a `DocumentSet` holds: frontmatter, title, tagline, summary and `##` sections, each read by a `ProseQuery` traversal. Its frontmatter is beet's `DocMeta`, carrying `created`, `updated` and `authors`, a Word file's from its core properties. |
| data block | `DataBlock`, `BlockFormat`, `Column`, `ColumnKind` | A fenced `csv` or `json` block named in its info string, defined once across the documents. |
| ask | `Ask`, `AskKind` | An open question to the owner, inline as `TODO(ask ...)`: a **fact** a source could answer, offered with a default, or a **decision** only the owner can make, challenged by the coach instead. |
| check kind | `CheckRoutes`, `DocumentCheck`, `FrontmatterCheck`, `H1Check`, `TaglineCheck`, `SummaryCheck`, `AgreesCheck`, `AsksCheck`, `SectionsCheck`, `BlockCheck`, and the params of each, `DocumentCheckParams`, `FrontmatterCheckParams`, `H1CheckParams`, `TaglineCheckParams`, `SummaryCheckParams`, `AgreesCheckParams`, `TitlePart`, `AsksCheckParams`, `SectionsCheckParams`, `BlockCheckParams` | The document check kinds, one route each under `check/`, mounted by `<CheckRoutes/>`. |
| render spec | `RenderSpec` | How a reader suite renders one form: the reader's own blank form, a Word file or a workbook, the output's name, and the form's quirks. |
| answers | `AnswerSheet`, `CellAnswer`, `AnswerApproval`, `AnswerState`, `FormStage` | A form's answers as markdown, one `## <cell>` section per cell and an `## Edits` list, written by the builder before any of the form's XML, reviewed with `eval/answers` and approved by a person with `eval/approve`, the approval holding the sheet as approved so a changed sheet is unapproved again. |
| build | `FormBuild`, `CellRef`, `FormCell`, `FormCellQuery`, `CellsReport` | One form's build, once its answers are approved: the blank form unpacked into the build directory as its suite's parts, which the builder edits as the form's own XML, zipped back into the built form, and its cells named by place, `<table>/<x>,<y>`, the table `t<n>` or a worksheet's name, as the builder reads them back. |
| deck triage | `DeckTriage`, `SlideTriage`, `PictureTriage`, `DeckTriageReport` | A source deck's slides sorted by what carries them, words or a picture, with every picture's native size and cover and what to watch for while transcribing it: hidden slides, recurring pictures, same-size pictures and exact duplicates. |
| claim | `Claim`, `Evidence`, `Stakes`, `ClaimStatus` | A row of the workspace suite's claims register: something the plan rests on that might be false, with its evidence on the ladder 0 asserted, 1 heard, 2 observed, 3 committed, its stakes, its test and its status. The coach holds the one `Claim::pick` picks: highest stakes, then least evidence, then open first. |
| action | `CoachAction` | One of the coach's moves, a row of a document suite's actions table: the ask, when to stop pushing, the red flags, and a bad and a good exchange. |
| verb | `EvalRoutes`, `EvalCheck`, `EvalResults`, `EvalNext`, `EvalBlocks`, `EvalWorksheet`, `EvalProject`, `EvalNew`, `EvalGrade`, `EvalRows`, `EvalPut`, `EvalDrop`, `EvalCells`, `EvalAnswers`, `EvalApprove`, `EvalBuild`, `EvalDeck` | One route of `beet-eval` under `eval/`, mounted by `<EvalRoutes/>`. |
| report | `ResultsReport`, `NextReport`, `WorksheetReport`, `WorksheetDocument`, `WorksheetSection`, `BlocksReport`, `BlockDump`, `BlockEntry`, `RowsReport`, `TableRow`, `ProjectionReport`, `Projection`, `ProjectedHeading`, `ProjectedCitation`, `CheckReport`, `CheckSummary`, `ScaffoldDocument`, `ScaffoldSection`, `ScaffoldBlock` | A verb's answer as a scene, a `#[template]` of HTML nodes beet renders as the request accepts, beside the data it shows, which a serde `Accept` answers instead; a file a verb writes is the scene's markdown. |
| clerk, grader, builder, coach | `EvalToolset`, `RoleSeed`, `NextUnit`, `HoldClaim`, `AssignmentWritten` | The four agent roles, below. |

## Layout

A suite and a workspace are stores, addressed by uri and never assumed to be on disk. Every structured thing in them is a row of a table or a typed document; prose for people, the documents under evaluation and the READMEs, stays markdown.

```
<suite>/
  suite.json            SuiteManifest
  README.md             what the suite is and provides, for a person
  outline.json          Outline, a document suite's
  seeds/<role>.md       the clerk's, grader's, builder's and coach's procedures, a document suite's
  evals/<id>            Eval rows
  rubrics/<id>          Rubric rows
  render/<rubric>       RenderSpec rows, a reader suite's
  actions/<id>          CoachAction rows, a document suite's
  claims/<id>           Claim rows, a workspace suite's
  assets/               the files the suite provides or was built from

<workspace>/
  workspace.json        Workspace
  docs/                 the documents under evaluation, markdown or Word
  results/grades/<id>   Grade rows, keyed on the eval
  results/approvals/<suite>/<rubric>  AnswerApproval rows, keyed on the rubric
  results/summary.json  Results, the last run
  results/projections/  the briefs eval/project writes
  dist/<suite>/         builds: each form's <rubric>.answers.md, its parts under <rubric>/, the built form, its <rubric>.cells.md
  suites/<name>/        the workspace's own suite, its claims register, its loop's threads under assets/threads/, the loop's journal under loop/
```

A table is one blob per row at `<table>/<key>`, the key being the row's id verbatim with no extension, pretty-printed JSON, so a tool call writing one row touches one object and a change diffs by field. The JSON is serde's default form of the Rust type: fields by name in sorted order, enum variants by their Rust name (`"Generic"`, `{"Binary": {...}}`), a newtype as its inner value, an absent option as `null`. The schema is derived from the same type by `ValueSchema::of`, so a form can be generated for each and the stored form validates against it. Every example below is tested to deserialize into its type, validate against its schema and serialize back unchanged.

### A suite

`suite.json`, the manifest. A source key is a tag, or a family of tags with its varying parts in angle brackets; a reference to the literature is a `ref:<slug>` entry.

```json SuiteManifest
{
  "builds_on": [],
  "kind": "Document",
  "name": "acme_biz",
  "outline": "outline.json",
  "sources": {
    "acme:outline": "This suite's own outline.json.",
    "mc:<topic>/<slug>": "A masterclass recording, by topic and slug.",
    "ref:sba": "US Small Business Administration, Write your business plan: the traditional outline the documents follow."
  }
}
```

`outline.json`, a document suite's outline. It generates, in the `structure` namespace, `<document>-present` and `<document>-sections` for each document, `<block>-block` for each block, and `frontmatter-complete` and `summaries-present` over the set.

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

`evals/structure.index-tagline-agreed`, a checked eval the suite writes itself.

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

`rubrics/01-business-plan`, a reader suite's rubric. A section's `sections` nest the form's subheadings.

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

`render/01-business-plan`, a reader suite's render spec.

```json RenderSpec
{
  "form": "assets/forms/01-business-plan.docx",
  "notes": "Tick the applicable boxes with Check by label, delete every red instruction sentence, and write in the third person.",
  "output": "01-business-plan.docx",
  "rubric": "01-business-plan"
}
```

`actions/status-quo`, one of a document suite's coach actions.

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

`claims/buyers-pay-monthly`, a row of a workspace suite's claims register.

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

`workspace.json`. Its own suite sits in the workspace store; any other is a store uri, a relative `fs:` path resolving against the directory the workspace runs from. A suite kept in a private bucket is an `S3` store, read with the pair the workspace entry's `<Secrets/>` opens from its sealed `secrets.toml`.

```json Workspace
{
  "dist": "dist",
  "docs": "docs",
  "suites": [
    { "Local": "suites/acme" },
    { "Store": { "Fs": { "path_prefix": "../beet_eval/suites/acme_biz" } } }
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

`results/approvals/acme_course/01-business-plan`, the approval of a form's answers, the sheet's text as approved.

```json AnswerApproval
{
  "answers": "## t0/1,0\n\nAcme Stalls\n",
  "by": "ada",
  "date": "2026-11-22",
  "rubric": "acme_course/01-business-plan"
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
      "decisions": 1,
      "document": "brand",
      "graded": 0,
      "judged": 9,
      "owner_gap": 21,
      "present": true,
      "reader_gap": 4,
      "shape_failures": ["structure.index-tagline-agreed"],
      "stale": false,
      "unclaimed": 0,
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

## The verbs

Each is a route of `beet-eval`, taking its flags in its signature so `--help` documents them and the strict cli server refuses any other, resolving the workspace through the store above it. A workspace runs them from its own entry, `eval.bsx` at its root, mounting `<EvalRoutes/>` under `eval`, `<CheckRoutes/>` under `check` and beet's `<DocumentView/>`, so the entry's directory, the repo store, is the workspace store; `tests/fixtures/acme/eval.bsx` is one:

```sh
beet-eval --entry=eval.bsx eval/next
```

A report answers a scene rendered as the request accepts: ANSI in a terminal, markdown when piped, ie to an agent or a file, HTML in a browser, or with `--accept=application/json` the stored form of what it computed. A verb that finds something wrong answers it and exits 1.

- `eval/check [--unused]`: every table's rows, global ids, the prose rules, each check's params against its route's, citations, and every anchor, action's served section and claim's block against the outline; `--unused` lists the evals no rubric cites.
- `eval/results`: runs every check, merges the grades, reads every rubric, writes `results/summary.json`, and answers the run, its line first.
- `eval/next`: the next step, with the document and rubric tables behind it.
- `eval/blocks [<name>]`: lists the data blocks, every block in full as JSON, or prints one's body as text.
- `eval/worksheet [<document>]`: every judged eval anchored in each document, with its statement and level lines.
- `eval/project <suite>/<rubric>`: the brief for one form, each heading's evals, the text at their anchors and its structural lines, answered and written to `results/projections/`.
- `eval/new <name> <author>`: scaffolds `docs/` from the outline, every body an ask; refuses a non-empty `docs/`.
- `eval/grade --eval --level --by [--anchor] [--evidence]`: writes one grade, refused unless the level is one the eval admits, at most 1 where an ask is open, and the evidence is verbatim from its anchor.
- `eval/rows --suite --table [--key]`: a table's rows, or one row by its key, each with its fields, and for a serde `Accept` the rows in their stored form; how the claims register is read.
- `eval/put --suite --table (--row | --from)`: writes one row of a suite's table, `evals`, `rubrics`, `render`, `actions` or `claims`, given inline as its JSON text (an agent's tool call passes the object, sent as that text), or drafted into the workspace store and named by its path, refused unless it keeps the law `eval/check` holds every row to.
- `eval/drop --suite --table --key`: removes one row of a suite's table, refused while a rubric cites the eval or a render spec builds the rubric.
- `eval/cells <suite>/<rubric>`: the cells of a reader's blank form by place, `t0/1,0` for the first table's second cell of its first row, `Start Here/3,2` for a workbook's, the map a builder writes against.
- `eval/answers <suite>/<rubric>`: reviews the form's answers in `dist/<suite>/<rubric>.answers.md`, refusing a section naming a cell the blank form lacks or holding no answer: each answer under its cell's blank text, the empty cells unanswered, the edits, the rubric's structural lines with their addresses to read the sheet against, and whether a person has approved it.
- `eval/approve <suite>/<rubric> --by=<who>`: records the answers as approved as they stand, the person who reviewed them naming themself; no role's toolset carries it.
- `eval/build <suite>/<rubric> [--fresh]`: refused until the form's answers are approved as they stand; unpacks the reader's blank form into `dist/<suite>/<rubric>/` the first time, or afresh with `--fresh`, zips those parts as the builder edited them into the built form, refusing parts that no longer read, and writes its cells beside it.
- `eval/deck <path>`: the triage of the slide deck at `path` in the store, read beside `view`'s dump before transcribing it, slides counted from 0 as the card stack counts them.
- `check/<kind>`: one document check kind, called by `eval/results` for every checked eval and directly for one, answering a `CheckVerdict`.
- `view <path> [--accept=text/markdown]`: beet's view of any file in the store, parsed as its media type and rendered as the request accepts, so a Word file, a workbook or a slide deck reads as markdown: headings, lists, GFM tables, checkboxes as `[ ]` and `[x]`, and a highlight as `<mark>`; a colour is no part of markdown, so a red instruction reads as plain text.

## The roles

The loop is routes, a workspace's `iterate.bsx`, and its run is a thread: the `iterate` route builds `LoopThread`, one Loop actor taking one unit of work a turn, `<NextUnit/>` asking `eval/next` for it, appending the decision to the loop thread as a post, `<verb> <target>: <why>`, and requesting it of the workspace's router as `<verb>/<first target>`, ie `write/product` or `build/acme_course/01-business-plan`, until `next` says `done` or names a coach's sitting, which only a person can hold, so the loop stops there and says which claim to sit down with. Its thread mounts one fixed path, `<Thread {(Sequence, MountThreadStore{path:"loop"})}>`, so every run continues one loop thread, the workspace's journal of decisions, each run a session of its Loop actor, opened by its join. The clerk, the grader and the builder are each a route building a thread for the unit it is asked (`<Route path="write/:document" {ThreadRoute{source:Document("templates/ClerkThread.bsx"), message:"write {document}"}}/>`), so a role also runs by hand, and a verb with no route stops the loop with the router's not-found. A role's thread is a template whose agent holds its tools in a `<Router>` of its own: `<EvalToolset/>`, the verbs `eval/check`, `eval/results`, `eval/next`, `eval/worksheet`, `eval/project`, `eval/blocks`, `eval/grade`, `eval/cells`, `eval/answers` and `eval/build` as beet `<RouteTool>`s, each named as a person runs it and forwarding to the workspace's verb, and beet's store tools. Its seed is its procedure, `<RoleSeed role="clerk"/>` reading `seeds/clerk.md` of the document suite, the initial posts of a fresh thread, so a continued thread keeps the seed it started with and an edited seed reaches a new thread (`--new`); each request posts the unit it asks, its route's `message` filled from the path param, once its actors have joined, so a continued thread is asked again. A seed's `{param}`s are filled from the request creating the thread, ie the coach's `{claim}`. Which stored thread a request continues is the path its thread mounts, filled from the request: the clerk continues its conversation about one document, `clerk/{document}`, the builder about one rubric and the coach about one claim, `coach/{claim}`, while the grader's mount is `new:true` and judges fresh every time, unanchored by its earlier grades. Threads relate by records: the loop requests each unit with `--spawned-by` naming its decision post, so a unit thread records the decision that created it and each join of one names the decision that asked for it, while a unit run by hand names none. Every thread persists as rows of the table store the entry declares above its routes, `<Router {JsonTableStore{path:"suites/<name>/assets/threads"}}>`, one json file per thread under the workspace suite's `assets/threads`, listed under its mount path. `tests/fixtures/acme/iterate.bsx` is one, its role threads under `templates/`, and `coach.bsx` the coach's sitting, every agent scripted:

```sh
beet-eval --entry=iterate.bsx iterate
beet-eval --entry=iterate.bsx grade product
beet-eval --entry=coach.bsx --claim=per-weekend
```

- **Clerk**: writes a document from its sources and the owner's answers, drafting what the sources allow and asking what only the owner knows. A decision ask whose section no claim names is raised as a claim through `eval/put` on the claims register, which `next` sends it to do before anything is coached.
- **Grader**: grades one document a turn against the worksheet through `eval/grade`, its store tools `read_only`, so its router has no `write-text` and it can read the workspace and never write it.
- **Builder**: fills a reader's form in two sittings. The first writes the brief's answers as markdown, one section per cell, into the build directory, the one store it can write, and reads them against the structural lines; a person reviews and approves them. The next writes the approved answers into the form's own XML, its parts unpacked beside them, so the built form looks as its author made it, and reads the built form's cells against the structural lines.
- **Coach**: holds one claim a sitting with the owner, a conversation rather than a unit of work, so it is no route: a workspace's `coach.bsx` holds the sitting's thread with `{RunThread}` under a surface to type into, entered with the start request, `beet-eval --entry=coach.bsx --claim=<claim id>`, its mount `coach/{claim}` filled from it, so each claim's sittings continue one thread about it. `next` names a sitting when the register holds a fatal claim at evidence 0 or 1. Its seed is its procedure, `seeds/coach.md` naming the claim held and the phrases it never says, then the answers of `eval/rows` for the actions table, the register and the claim held, `<RoutePost path="eval/rows?suite=..&table=claims&key={claim}"/>`, whose `test` is the last sitting's assignment. It asks one question a turn, gives a decision no default, keeps the register through `eval/put` pinned to the claims table, `<RouteTool path="eval/put" params="suite=..&table=claims"/>`, and writes its notes under the workspace suite's `assets/`. The sitting is control flow: `<HoldClaim/>`, reading the claim the sitting was entered with and the `test` it opens with, then a `<Repeat>` of the coach's turn, `<AssignmentWritten/>` and the owner's turn, under a `<Fallback>` ending in `<EndWith/>`, so the assignment written to the claim's `test` ends it as a pass.
