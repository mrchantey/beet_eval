use crate::prelude::*;
use beet::prelude::*;

/// Where the subject needs work next: one unit of work, why, and the tables the
/// choice was made from. `eval/next` computes it from the same run as
/// [`Results`], in this order:
///
/// 1. [`Scaffold`](NextVerb::Scaffold) when no document of the outline exists.
/// 2. [`Write`](NextVerb::Write) the first missing document, then the document
///    failing the most shape checks: shape before substance.
/// 3. [`Grade`](NextVerb::Grade) every document written with no asks open but
///    ungraded, or graded before its last `updated`.
/// 4. [`Build`](NextVerb::Build) the first reader rubric fully met, with a
///    [`RenderSpec`] and nothing built for it.
/// 5. [`Write`](NextVerb::Write) the document with the most open asks and the
///    widest gap to the owner rubric, weighing an ask as two levels.
/// 6. [`Done`](NextVerb::Done) when nothing is left.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct NextStep {
	/// The unit of work.
	pub verb: NextVerb,
	/// What it is done to: a document, the documents to grade, a
	/// `<package>/<rubric>` to build, a claim to coach, `docs/` to scaffold;
	/// empty when done.
	pub targets: Vec<SmolStr>,
	/// Why this unit and not another, one paragraph.
	pub why: String,
	/// Every document of the outline, in its order.
	pub documents: Vec<DocumentRow>,
	/// Every rubric of every package.
	pub rubrics: Vec<RubricRow>,
}

impl NextStep {
	/// The triage over one run: the documents of the outline in order, each
	/// weighed by its asks, its shape failures, its gaps to the owner and the
	/// reader rubrics and its grading, then the unit of work by the order
	/// above. `built` names the rubrics whose form has a cells dump in the
	/// build directory.
	pub fn compute(
		workspace: &LoadedWorkspace,
		documents: &DocumentSet,
		verdicts: &[(EvalId, CheckVerdict)],
		results: &Results,
		built: &[RubricRef],
	) -> Result<Self> {
		let document_package =
			workspace.document_package()?.manifest.name.clone();
		let outline = workspace.outline()?;
		let docs = &workspace.manifest.docs;
		let judged = workspace
			.evals
			.iter()
			.filter(|packaged| !packaged.eval.is_checked())
			.map(|packaged| &packaged.eval)
			.collect::<Vec<_>>();
		let anchored_in = |eval: &Eval, name: &str| {
			eval.anchor_or_namespace().document_name() == name
		};
		let documents_rows = outline
			.documents
			.iter()
			.map(|spec| {
				let name = spec.name.as_str();
				let document = documents.get(name);
				let mut owner_gap = 0;
				let mut reader_gap = 0;
				for rubric in &results.rubrics {
					for citation in &rubric.citations {
						let Some(eval) =
							judged.iter().find(|eval| eval.id == citation.eval)
						else {
							continue;
						};
						if !anchored_in(eval, name) {
							continue;
						}
						let gap = match citation.status {
							CitationStatus::Met => 0,
							CitationStatus::Awaiting => {
								citation.level.get() as u32
							}
							CitationStatus::Failing { got } => {
								citation.level.get().saturating_sub(got.get())
									as u32
							}
						};
						match rubric.id.package() == document_package {
							true => owner_gap += gap,
							false => reader_gap += gap,
						}
					}
				}
				let judged_here = judged
					.iter()
					.filter(|eval| anchored_in(eval, name))
					.collect::<Vec<_>>();
				let graded = judged_here
					.iter()
					.filter(|eval| {
						results.grades.as_ref().is_some_and(|grades| {
							grades.get(&eval.id).is_some()
						})
					})
					.count() as u32;
				let updated = document
					.and_then(|document| document.meta.as_ref())
					.and_then(|meta| meta.updated);
				let stale = graded > 0
					&& match (&results.grades, updated) {
						(Some(grades), Some(updated)) => updated > grades.date,
						_ => false,
					};
				DocumentRow {
					document: spec.name.clone(),
					present: document.is_some(),
					asks: documents
						.asks()
						.iter()
						.filter(|(document, _)| {
							DocumentSet::top_level(document) == name
						})
						.count() as u32,
					shape_failures: verdicts
						.iter()
						.filter(|(_, verdict)| {
							!verdict.pass
								&& verdict
									.documents
									.iter()
									.any(|document| document == name)
						})
						.map(|(id, _)| id.clone())
						.collect(),
					owner_gap,
					reader_gap,
					graded,
					judged: judged_here.len() as u32,
					updated,
					stale,
				}
			})
			.collect::<Vec<_>>();
		let rubric_rows =
			results.rubrics.iter().map(RubricResult::row).collect();
		let (verb, targets, why) = Self::choose(
			workspace,
			docs,
			&documents_rows,
			results,
			built,
			&document_package,
		);
		Self {
			verb,
			targets,
			why,
			documents: documents_rows,
			rubrics: rubric_rows,
		}
		.xok()
	}

	/// The unit of work, its targets and why, in the order the type
	/// documents.
	fn choose(
		workspace: &LoadedWorkspace,
		docs: &RelPath,
		rows: &[DocumentRow],
		results: &Results,
		built: &[RubricRef],
		document_package: &str,
	) -> (NextVerb, Vec<SmolStr>, String) {
		let absent = rows.iter().filter(|row| !row.present).collect::<Vec<_>>();
		if absent.len() == rows.len() {
			return (
				NextVerb::Scaffold,
				vec![format!("{docs}/").into()],
				format!(
					"{docs}/ is empty: `eval/new <name> <author>`, then a sitting on \
					 the index."
				),
			);
		}
		if let Some(first) = absent.first() {
			return (
				NextVerb::Write,
				vec![first.document.clone()],
				format!(
					"{docs}/{} does not exist, and `eval/new` refuses a non-empty \
					 {docs}/, so it is scaffolded by hand from the outline's sections.",
					first.document
				),
			);
		}
		let mut shaped = rows
			.iter()
			.filter(|row| !row.shape_failures.is_empty())
			.collect::<Vec<_>>();
		shaped.sort_by_key(|row| std::cmp::Reverse(row.shape_failures.len()));
		if let Some(row) = shaped.first() {
			return (
				NextVerb::Write,
				vec![row.document.clone()],
				format!(
					"{docs}/{} fails {} shape check(s): {}. Shape before substance.",
					row.document,
					row.shape_failures.len(),
					row.shape_failures
						.iter()
						.map(EvalId::to_string)
						.collect::<Vec<_>>()
						.join(", ")
				),
			);
		}
		let to_grade =
			rows.iter()
				.filter(|row| {
					row.asks == 0
						&& row.judged > 0 && (row.graded < row.judged || row.stale)
				})
				.collect::<Vec<_>>();
		if !to_grade.is_empty() {
			let why = to_grade
				.iter()
				.map(|row| match row.stale {
					true => format!(
						"{} is graded before its last edit on {}",
						row.document,
						row.updated
							.map(|date| date.to_string())
							.unwrap_or_default()
					),
					false if row.graded > 0 => format!(
						"{} is written with no asks open and {} of {} evals ungraded",
						row.document,
						row.judged - row.graded,
						row.judged
					),
					false => format!(
						"{} is written with no asks open and ungraded",
						row.document
					),
				})
				.collect::<Vec<_>>()
				.join("; ");
			return (
				NextVerb::Grade,
				to_grade.iter().map(|row| row.document.clone()).collect(),
				format!("{why}."),
			);
		}
		let ready = results
			.rubrics
			.iter()
			.filter(|rubric| {
				let row = rubric.row();
				rubric.id.package() != document_package
					&& row.failing == 0
					&& row.awaiting == 0
					&& workspace
						.package(rubric.id.package())
						.and_then(|package| {
							package.render_spec(rubric.id.rubric())
						})
						.is_some() && !built.contains(&rubric.id)
			})
			.collect::<Vec<_>>();
		if let Some(first) = ready.first() {
			let more = match ready.len() {
				1 => String::new(),
				count => format!("; {} more form(s) are ready too", count - 1),
			};
			return (
				NextVerb::Build,
				vec![first.id.as_str().into()],
				format!(
					"every citation of {} is met and nothing is in {}/ for it{more}.",
					first.id, workspace.manifest.dist
				),
			);
		}
		let mut scored = rows
			.iter()
			.map(|row| (row, row.asks * 2 + row.owner_gap))
			.collect::<Vec<_>>();
		scored.sort_by_key(|(_, score)| std::cmp::Reverse(*score));
		match scored.first() {
			Some((row, score)) if *score > 0 => {
				let reader = match row.reader_gap {
					0 => String::new(),
					gap => format!(", reader gap {gap}"),
				};
				(
					NextVerb::Write,
					vec![row.document.clone()],
					format!(
						"{} ask(s) open and an owner gap of {}{reader}, the most of any \
						 document.",
						row.asks, row.owner_gap
					),
				)
			}
			_ => (
				NextVerb::Done,
				Vec::new(),
				"no asks open, every owner citation met, every ready form built.".into(),
			),
		}
	}

	/// The step as the triage prints it: the unit of work and why, then the
	/// document and rubric tables.
	pub fn to_markdown(&self) -> String {
		let target = match self.targets.is_empty() {
			true => String::new(),
			false => format!(" {}", self.targets.join(", ")),
		};
		let mut out = vec![
			format!("## Next: {}{target}", self.verb.word()),
			String::new(),
			self.why.clone(),
			String::new(),
			"## Documents".into(),
			String::new(),
			"| Document | Present | Asks | Shape failures | Owner gap | Reader gap | Graded | Updated |"
				.into(),
			"|---|---|---|---|---|---|---|---|".into(),
		];
		for row in &self.documents {
			out.push(format!(
				"| {} | {} | {} | {} | {} | {} | {}/{}{} | {} |",
				row.document,
				if row.present { "yes" } else { "no" },
				row.asks,
				row.shape_failures.len(),
				row.owner_gap,
				row.reader_gap,
				row.graded,
				row.judged,
				if row.stale { " stale" } else { "" },
				row.updated.map(|date| date.to_string()).unwrap_or_default()
			));
		}
		out.extend([
			String::new(),
			"## Rubrics".into(),
			String::new(),
			"| Rubric | Met | Failing | Awaiting |".into(),
			"|---|---|---|---|".into(),
		]);
		for row in &self.rubrics {
			out.push(format!(
				"| {} | {} | {} | {} |",
				row.rubric, row.met, row.failing, row.awaiting
			));
		}
		format!("{}\n", out.join("\n"))
	}
}

/// The kinds of unit of work, each a role's turn.
#[derive(
	Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize,
)]
pub enum NextVerb {
	/// `eval/new` lays out the documents from the outline, every body an ask.
	Scaffold,
	/// The clerk writes a document from the sources and the owner's answers.
	Write,
	/// The grader grades documents against the worksheet.
	Grade,
	/// The builder fills a reader's form.
	Build,
	/// The coach runs a sitting on a claim.
	Coach,
	/// Nothing is left.
	Done,
}

impl NextVerb {
	/// The verb as the triage prints it, ie `write`.
	pub fn word(&self) -> &'static str {
		match self {
			Self::Scaffold => "scaffold",
			Self::Write => "write",
			Self::Grade => "grade",
			Self::Build => "build",
			Self::Coach => "coach",
			Self::Done => "done",
		}
	}
}

/// One document as the triage weighs it.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct DocumentRow {
	/// The document's name in the outline.
	pub document: SmolStr,
	/// Whether it exists, as a file or a directory with an index.
	pub present: bool,
	/// The asks open in it and its children.
	pub asks: u32,
	/// The failing checks that concern its shape.
	pub shape_failures: Vec<EvalId>,
	/// The levels still missing to the owner rubric, over the evals anchored
	/// here.
	pub owner_gap: u32,
	/// The same over every reader rubric.
	pub reader_gap: u32,
	/// How many of the judged evals anchored here are graded.
	pub graded: u32,
	/// How many judged evals are anchored here.
	pub judged: u32,
	/// Its frontmatter's `updated`.
	pub updated: Option<Date>,
	/// Whether it changed after its grades were given.
	pub stale: bool,
}

/// One rubric as the triage counts it.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct RubricRow {
	/// The rubric, `<package>/<rubric>`.
	pub rubric: RubricRef,
	/// Citations met.
	pub met: u32,
	/// Citations failing.
	pub failing: u32,
	/// Citations awaiting a grader.
	pub awaiting: u32,
}
