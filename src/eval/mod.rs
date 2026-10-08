//! The core, agnostic of the subject: evals, the scale, checks, rubrics,
//! grades, results and the triage, and the packages and workspaces that hold
//! them. `README.md` fixes the words; this is the law they obey.
//!
//! # Evals
//!
//! An [`Eval`] is one statement about the subject that can be found true or
//! false of it, defined exactly once as a row of a package's `evals` table.
//! Two kinds share the row: a **judged** eval, graded on the four-level scale
//! by a grader, and a **checked** eval, decided by the check route its
//! [`Levels::Binary`] names. A [`Rubric`] is the third thing, a profile citing
//! evals at the level one reader demands, plus the structural lines of that
//! reader's form. The eval is the test, a rubric the suite run for one
//! audience, and one run over the subject produces one [`Results`] every rubric
//! reads.
//!
//! A document package's **owner rubric** holds the documents to level 3 on what
//! matters to the subject, and is the default every workspace runs. A reader
//! package's rubrics hold the same documents to what one outside reader
//! accepts, mostly level 2, and add the structural lines of that reader's
//! forms. Judged evals are always graded on `docs/`, never on a rendered form:
//! a form asking for one line on pricing still holds the subject to the whole
//! pricing eval at the level its rubric names, and only the form's structural
//! lines are checked on the rendering.
//!
//! ## Ids
//!
//! An [`EvalId`] is `namespace.slug`, global across the workspace, so two
//! packages may not both define one; `eval/check` refuses it. A document
//! package names an eval's namespace after the document its anchor lies in,
//! and keeps `structure` for the checks on the shape of the set, part of which
//! its [`Outline`](crate::prelude::Outline) generates.
//!
//! An id is permanent: to change what an eval means, add a new one, delete the
//! old and fix every rubric that cited it. A move between namespaces when a
//! package's documents change is the one exception: the slug keeps its
//! meaning, every citation is rewritten in the same change, and the package's
//! README records the move.
//!
//! ## The scale
//!
//! Every judged eval is graded on the same four [`EvalLevel`]s, whose words
//! apply unless the eval pins its own. A rubric names the level it demands.
//!
//! | Level | Meaning |
//! |---|---|
//! | 0 | Missing: nothing is written, or only a placeholder, a category or a restated prompt. |
//! | 1 | Stated: an answer exists but is generic, unsupported or incomplete; it could belong to any subject. |
//! | 2 | Sound: specific to this subject, complete, with a reason or a source; what an assessor or a coach accepts. |
//! | 3 | Strong: evidenced, quantified and tested against alternatives; what an investor, a partner or the author a year later would hold it to. |
//!
//! A binary eval is met or not, 0 or 2: a check awards 2 on a pass, a grader
//! gives a judged binary eval one or the other.
//!
//! ## The fields
//!
//! - **statement**: exactly one sentence ending in a full stop, true of a
//!   strong subject and false of a missing one. No count that belongs to a
//!   form: "several" or "each", not "five", unless the number is intrinsic.
//! - **levels**: [`Levels::Generic`] when the scale applies as written,
//!   [`Levels::Binary`] when met or not, [`Levels::Custom`] when the scale's
//!   words are not precise enough, pinning all four lines, each one sentence
//!   ending in a full stop and strictly harder than the last.
//! - **check**: only on a binary eval, a [`CheckRef`] naming a route under
//!   `check/` and its params. The route decides; the statement still says in
//!   words what it decides.
//! - **sources**: one or more [`SourceTag`]s, most authoritative first. A
//!   tag's meaning is in the source table of the package that owns the source;
//!   `ref:<slug>` points at an entry in the citing package's own table, and
//!   `inferred` is a legal source and an honest one.
//! - **anchor**: the [`Address`] where the statement is expected to be
//!   satisfied, which a grader, a worksheet and a projection point at. A
//!   document package sets it on every judged eval; absent, it is the document
//!   the namespace names.
//! - **note**: one optional paragraph for a contradiction between sources, an
//!   accepted not-applicable answer, or a warning to the grader. Never a
//!   second statement.
//!
//! Prose in every field: no em dashes, no line breaks inside a sentence, one
//! eval per idea and one idea per eval.
//!
//! ## Check kinds
//!
//! A check kind is a route under `check/` in the engine's router, its params a
//! [`Reflect`](beet::prelude::Reflect) type behind its `ParamsPartial`, against
//! which `eval/check` validates every [`CheckRef`] without calling it. A run
//! calls the route with the ref's params as flags, [`CheckRef::call`], and
//! reads the [`CheckVerdict`] it answers. A new kind is a new route answering
//! a verdict, nothing else. The document kinds are listed in the
//! [`checks`](crate::docs::checks) module.
//!
//! ## Rubrics
//!
//! A rubric names its reader, introduces the form, lists its sources in order
//! of authority, then follows the form's own headings, each a
//! [`RubricSection`] with what the reader looks for and two lists.
//!
//! - A [`Citation`] is an eval at a level, 1 to 3, and 2 only for a binary
//!   eval. The section's prose says why; the eval says what. `eval/check`
//!   refuses an unknown id or a level the eval cannot reach.
//! - A [`StructuralLine`] is one checkable sentence about the form with its
//!   source tags: a cell filled, a count of rows, a sentence deleted, a mark
//!   made bold. It is checked on the rendered form, never on `docs/`, the
//!   mirror image of a citation. Its address is `label.n`, the section's label
//!   and its position, so a result can cite it: append to a list, never
//!   insert.
//!
//! A requirement that one document match another is neither: it goes in the
//! section's `agrees_with`.
//!
//! ## What is not an eval
//!
//! - **Agreement**: one rendering carrying the same fact as another. When
//!   every rendering draws from the same section of `docs/`, agreement is a
//!   property of the pipeline and is checked as one.
//! - **Set-level quality**: duplication across documents, contradictions
//!   between sections, stale figures. These are checks over the whole of
//!   `docs/` in their own pass.
//!
//! ## Grades and results
//!
//! A grader works from `eval/worksheet` and writes one [`Grade`] per judged
//! eval through `eval/grade`: the level by the eval's own lines, the lower of
//! two when between them; the anchor where the evidence sits; a verbatim quote
//! of at most twenty-five words, or none at 0 when nothing is written. A
//! section with an ask open is graded at most 1 on every eval anchored there.
//! `eval/grade` refuses an unknown or checked eval, a level the eval does not
//! admit, and a quote that is not in the anchored section.
//!
//! `eval/results` runs every check, merges the grades, refusing a row whose
//! eval has since changed, and reads every rubric: a citation of a checked
//! eval is met when its check passes, of a judged eval when its grade reaches
//! the cited level, and awaiting when no grade exists. It writes
//! `results/summary.json`; `eval/next` triages from the same computation.
//!
//! ## Packages and workspaces
//!
//! A package is a store with a [`PackageManifest`], whose docs give the layout
//! and how its tables lie in the store; a workspace is a store with a
//! [`Workspace`] manifest naming its packages. A verb reads both once, a
//! [`LoadedWorkspace`]: every package with its rows, every eval of all of them
//! in one list, and what could not be read, which `eval/check` reports and
//! every other verb refuses to run on.
mod check;
mod eval;
mod grade;
mod levels;
mod next;
mod package;
mod results;
mod rubric;
mod workspace;
pub use check::*;
pub use eval::*;
pub use grade::*;
pub use levels::*;
pub use next::*;
pub use package::*;
pub use results::*;
pub use rubric::*;
pub use workspace::*;
