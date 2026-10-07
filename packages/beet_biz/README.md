# beet_biz

The document package for a business: the seven documents a business keeps, the evals every one of them is graded against, and the checks on the shape of the set. Its law is `beet_eval`'s module docs, `eval` for the evals and `markdown` for the documents, and it names no business.

## What it provides

- `package.json`: the manifest, with the source tags this package owns, the literature as `ref:<slug>` entries among them.
- `outline.json`: the seven documents, `index`, `product`, `market`, `brand`, `legal`, `finance` and `team`, their sections in order with what each holds, and the thirteen data blocks they carry with their typed columns. It generates the 29 `structure` evals that restate it: each document present and its sections, each block's home and columns, the frontmatter and the summaries over the set.
- `evals/`: 137 rows, the 132 judged evals across the six documents other than the index, whose few evals live in the `product` namespace and are anchored at `index`, and five checked ones in `structure` that the outline cannot generate: the index titled and taglined, the brand document agreeing with it on both, and no ask left open.
- `rubrics/owner`: the owner rubric, level 3 on what matters to running the business and 2 on the knowledge a course asks for, citing every eval in the package once; the default every business workspace runs.

## Scope by namespace

| Namespace | Scope |
|---|---|
| `product` | the idea, value proposition, products and services, pricing and unit economics, terms of sale, and the operation behind them: location, hours, contact, partners and activities, resources and suppliers, the action plan, travel |
| `market` | market research, size, trends, price points, supplier availability, special considerations, customers and their discovery, competitors, SWOT, competitive advantage, and marketing: objective, channels, sales channels and payment, promotion, calendar, budget, journey, review and results |
| `brand` | name and tagline, name clearance, positioning, archetypes, voice, identity, the consistency of the assets |
| `legal` | structure, registrations, GST, licences, insurance, WHS, risk, employer obligations, rights and contracts, conduct, and compliance once trading: the obligations register, checks, corrective action, records |
| `finance` | sources of finance, startup costs, cost structure, projections, cash flow, profit and loss, break even, personal budget and balance sheet, reporting, tax, working capital, debt collection |
| `team` | the people: reasons, expectations, commitment, skills and gaps, advice sought |
| `structure` | the checked evals: what the outline generates, and the index's title and tagline, the brand document's agreement with them and no open asks |

An eval that fits none of these is a reason to discuss the outline, not to add a namespace.

## Source tags

`biz:outline` is this package's own `outline.json`, and every `ref:<slug>` an entry of the manifest's sources naming the work and the idea taken from it. Every other tag an eval carries, `[template]`, `[guide bold]`, `[A2.3]`, `[mc:<topic>/<slug>]`, `[bdj-part-4]`, `[demo]` and the rest, is a source in the Sarina Russo course, defined in that reader package's manifest. The evals were extracted from that course's forms and recordings and keep their provenance; they are written so that any business can be held to them.

## The layout

The outline follows the business plan outline that lenders, grant bodies and coaches read by, as the US Small Business Administration and business.gov.au publish it: summary, company and offer, market, marketing and sales, operations, legal, finance, team. Three merges make it a handful: the offer and the operation that delivers it are one document, the market and the marketing into it are one, and the law and the keeping of it are one. Brand is lifted out of marketing into a document of its own, because a brand outlives any campaign. What the outline does not give is life after the plan is written; for that the package borrows the register from management systems: the price list, the risk register, the obligations register and the projections are tables defined once and maintained for as long as the business trades.

Reading order is the outline's order and the index declares it, so the files carry no numbers. The split is inside, outside and face: product is everything the business controls, market is everything it must understand and reach, brand is how the two meet; legal, finance and team are the three accounts every reader asks for. Any document may be promoted to a directory; the checks resolve either. A block's columns are expected to move once the first business is written against them, and a change is recorded in the outline's history.

## History

- 2026-09-28: 126 criteria extracted from the thirteen Sarina Russo rubrics into ten topic files: idea, market, customers, offer, marketing, operations, legal, compliance, finance, founder.
- 2026-10-02: the ten folded into six document files, slugs unchanged. Prefixes moved `idea`, `offer` and `operations` to `product`, `customers` and `marketing` to `market`, `compliance` to `legal` and `founder` to `team`, except `offer.sales-channels-listed` and `offer.payment-options-named`, which went to `market`, and `marketing.tag-line`, which went to `brand`.
- 2026-10-02: six brand evals added from the literature; seventeen checked evals on the shape of the set, thirteen on the data blocks and four on agreement with the index, summaries and asks; an anchor on every judged eval; the owner rubric written.
- 2026-10-07: the markdown package converted to tables: the template is `outline.json`, which generates the structure evals restating it, the tag `biz:template` is `biz:outline`, and `references.md` folded into the manifest's sources.
