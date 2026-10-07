use crate::prelude::*;
use beet::prelude::*;

/// One run over the subject: every check decided, the grades merged, and every
/// rubric read off the outcome. Written by `eval/results` to
/// `results/summary.json`, whose `--format=md` renders the same value for a
/// reader; never edited by hand.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct Results {
	/// The day of the run.
	pub ran_at: Date,
	/// Every checked eval's outcome, by id.
	pub checks: Vec<CheckOutcome>,
	/// The grades merged into the run, absent when no grader has written one.
	pub grades: Option<GradeSet>,
	/// Every rubric of every package, read off the checks and the grades.
	pub rubrics: Vec<RubricResult>,
}

impl Results {
	/// Where a workspace keeps the last run, below its results directory.
	pub const SUMMARY: &str = "summary.json";
	/// The most ungraded evals the rendering names.
	const UNGRADED_SHOWN: usize = 30;

	/// One run read off the checks and the grades: each checked eval's
	/// verdict, the grade rows merged, refusing one whose eval is unknown,
	/// checked, or no longer admits its level, and every rubric's citations
	/// read as met, failing or awaiting.
	pub fn compute(
		ran_at: Date,
		evals: &[PackagedEval],
		verdicts: &[(EvalId, CheckVerdict)],
		grades: Vec<Grade>,
		rubrics: &[(RubricRef, &Rubric)],
	) -> Self {
		let eval =
			|id: &EvalId| evals.iter().find(|packaged| &packaged.eval.id == id);
		let mut checks = verdicts
			.iter()
			.map(|(id, verdict)| CheckOutcome {
				eval: id.clone(),
				pass: verdict.pass,
				detail: verdict.detail.clone(),
			})
			.collect::<Vec<_>>();
		checks.sort_by(|left, right| left.eval.cmp(&right.eval));
		let grades = GradeSet::merge(grades, |grade| match eval(&grade.eval) {
			None => Some(format!("{}: unknown eval", grade.eval)),
			Some(packaged) if packaged.eval.is_checked() => Some(format!(
				"{} is a checked eval; its result comes from the check",
				grade.eval
			)),
			Some(packaged) if !packaged.eval.levels.admits(grade.level) => {
				Some(format!(
					"{} is binary and takes 0 or 2, not {}",
					grade.eval, grade.level
				))
			}
			Some(_) => None,
		});
		let rubrics = rubrics
			.iter()
			.map(|(reference, rubric)| RubricResult {
				id: reference.clone(),
				citations: rubric
					.citations()
					.map(|citation| CitationResult {
						eval: citation.eval.clone(),
						level: citation.level,
						status: Self::status(
							citation,
							&checks,
							grades.as_ref(),
						),
					})
					.collect(),
			})
			.collect();
		Self {
			ran_at,
			checks,
			grades,
			rubrics,
		}
	}

	/// A citation as the run reads it: a check passing meets level 2 and
	/// no more, a grade meets what it reaches, and a judged eval no grader has
	/// graded awaits.
	fn status(
		citation: &Citation,
		checks: &[CheckOutcome],
		grades: Option<&GradeSet>,
	) -> CitationStatus {
		if let Some(check) =
			checks.iter().find(|check| check.eval == citation.eval)
		{
			let got = match check.pass {
				true => EvalLevel::SOUND,
				false => EvalLevel::MISSING,
			};
			return match check.pass && citation.level <= EvalLevel::SOUND {
				true => CitationStatus::Met,
				false => CitationStatus::Failing { got },
			};
		}
		match grades.and_then(|grades| grades.get(&citation.eval)) {
			Some(grade) if grade.level >= citation.level => CitationStatus::Met,
			Some(grade) => CitationStatus::Failing { got: grade.level },
			None => CitationStatus::Awaiting,
		}
	}

	/// The run for a person, `--format=md`: every check with its detail, the
	/// state of the grades, and every rubric with its failing citations.
	/// `evals` gives the judged count and the ungraded list, `manifest` the
	/// directories the prose names.
	pub fn to_markdown(
		&self,
		evals: &[PackagedEval],
		manifest: &Workspace,
	) -> String {
		let judged = evals
			.iter()
			.filter(|packaged| !packaged.eval.is_checked())
			.map(|packaged| &packaged.eval.id)
			.collect::<Vec<_>>();
		let grades_table = format!("{}/grades", manifest.results);
		let passed = self.checks.iter().filter(|check| check.pass).count();
		let mut out = vec![
			"# Results".to_string(),
			String::new(),
			format!(
				"Written by `eval/results` on {} against `{}/`. A checked eval is \
				 decided here; a judged eval takes its level from the `{grades_table}` \
				 table when a grader has written one and is otherwise awaiting. Do \
				 not edit by hand.",
				self.ran_at, manifest.docs
			),
			String::new(),
			"## Checks".into(),
			String::new(),
			"| Eval | Result | Detail |".into(),
			"|---|---|---|".into(),
		];
		for check in &self.checks {
			out.push(format!(
				"| `{}` | {} | {} |",
				check.eval,
				if check.pass { "pass" } else { "fail" },
				check.detail.replace('|', "\\|")
			));
		}
		out.extend([
			String::new(),
			format!("{passed} of {} checks pass.", self.checks.len()),
			String::new(),
			"## Grades".into(),
			String::new(),
		]);
		match &self.grades {
			None => out.push(format!(
				"No grades in `{grades_table}`: the {} judged evals await a grader. \
				 The protocol is the `grade-docs` skill and the worksheet is \
				 `eval/worksheet`.",
				judged.len()
			)),
			Some(grades) => {
				let distribution = (0..=3)
					.map(|level| {
						grades
							.rows
							.iter()
							.filter(|grade| grade.level.get() == level)
							.count()
							.to_string()
					})
					.collect::<Vec<_>>()
					.join("/");
				out.push(format!(
					"Graded {} by {}: {} of {} judged evals have a level. \
					 Distribution 0/1/2/3: {distribution}.",
					grades.date,
					grades.by.join(", "),
					grades.rows.len(),
					judged.len()
				));
				let ungraded = judged
					.iter()
					.filter(|id| grades.get(id).is_none())
					.collect::<Vec<_>>();
				if !ungraded.is_empty() {
					let shown = ungraded
						.iter()
						.take(Self::UNGRADED_SHOWN)
						.map(|id| format!("`{id}`"))
						.collect::<Vec<_>>()
						.join(", ");
					let more = match ungraded.len() > Self::UNGRADED_SHOWN {
						true => format!(
							", and {} more",
							ungraded.len() - Self::UNGRADED_SHOWN
						),
						false => String::new(),
					};
					out.extend([
						String::new(),
						format!("Ungraded: {shown}{more}."),
					]);
				}
				if !grades.problems.is_empty() {
					out.extend([
						String::new(),
						format!("Problems in `{grades_table}`:"),
						String::new(),
					]);
					out.extend(
						grades
							.problems
							.iter()
							.map(|problem| format!("- {problem}")),
					);
				}
			}
		}
		out.extend([
			String::new(),
			"## Rubrics".into(),
			String::new(),
			"| Rubric | Citations | Met | Failing | Awaiting |".into(),
			"|---|---|---|---|---|".into(),
		]);
		let mut detail = Vec::new();
		for rubric in &self.rubrics {
			let row = rubric.row();
			out.push(format!(
				"| `{}` | {} | {} | {} | {} |",
				rubric.id,
				rubric.citations.len(),
				row.met,
				row.failing,
				row.awaiting
			));
			detail.extend([format!("### {}", rubric.id), String::new()]);
			for citation in &rubric.citations {
				if let CitationStatus::Failing { got } = citation.status {
					detail.push(format!(
						"- `{}` needs {}, {}",
						citation.eval,
						citation.level,
						self.failing_detail(&citation.eval, got)
					));
				}
			}
			if row.awaiting > 0 {
				detail.push(format!(
					"- {} citation(s) awaiting a grader.",
					row.awaiting
				));
			}
			if row.failing == 0 && row.awaiting == 0 {
				detail.push("All citations met.".into());
			}
			detail.push(String::new());
		}
		out.push(String::new());
		out.extend(detail);
		format!("{}\n", out.join("\n").trim_end())
	}

	/// Why a failing citation fails: its check, or the grade it reached.
	fn failing_detail(&self, eval: &EvalId, got: EvalLevel) -> String {
		match self.checks.iter().find(|check| &check.eval == eval) {
			Some(check) if check.pass => {
				"check passes but cannot reach 3".into()
			}
			Some(_) => "check fails".into(),
			None => {
				let anchor = self
					.grades
					.as_ref()
					.and_then(|grades| grades.get(eval))
					.map(|grade| format!(" at `{}`", grade.anchor))
					.unwrap_or_default();
				format!("graded {got}{anchor}")
			}
		}
	}

	/// The run's one line: checks passed, grades merged, rubrics read.
	pub fn line(&self, evals: &[PackagedEval]) -> String {
		let judged = evals
			.iter()
			.filter(|packaged| !packaged.eval.is_checked())
			.count();
		let passed = self.checks.iter().filter(|check| check.pass).count();
		let grades = match &self.grades {
			Some(grades) => {
				format!("{} of {judged} judged evals graded", grades.rows.len())
			}
			None => "no grades".into(),
		};
		format!(
			"{passed} of {} checks pass; {grades}; {} rubrics read",
			self.checks.len(),
			self.rubrics.len()
		)
	}
}

impl GradeSet {
	/// The grade rows merged into a set, a row `refuse` names a reason for
	/// listed rather than merged; `None` when there are no rows at all.
	pub fn merge(
		grades: Vec<Grade>,
		refuse: impl Fn(&Grade) -> Option<String>,
	) -> Option<Self> {
		if grades.is_empty() {
			return None;
		}
		let mut set = Self {
			date: grades.iter().map(|grade| grade.date).max()?,
			by: Vec::new(),
			rows: Vec::new(),
			problems: Vec::new(),
		};
		for grade in grades {
			match refuse(&grade) {
				Some(problem) => set.problems.push(problem),
				None => {
					if !set.by.contains(&grade.by) {
						set.by.push(grade.by.clone());
					}
					set.rows.push(grade);
				}
			}
		}
		set.by.sort();
		set.rows.sort_by(|left, right| left.eval.cmp(&right.eval));
		set.xsome()
	}

	/// The merged grade of `eval`.
	pub fn get(&self, eval: &EvalId) -> Option<&Grade> {
		self.rows.iter().find(|grade| &grade.eval == eval)
	}
}

/// The grades a run merged: the rows as they stood, and the rows it could not
/// use because the eval they grade has since been deleted, made checked, or
/// lost the level they give.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct GradeSet {
	/// The latest day any merged row was given.
	pub date: Date,
	/// Every grader of record among the rows, sorted.
	pub by: Vec<SmolStr>,
	/// The merged rows, by eval.
	pub rows: Vec<Grade>,
	/// Why each refused row was refused.
	pub problems: Vec<String>,
}

/// One rubric as a run reads it.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct RubricResult {
	/// The rubric, `<package>/<rubric>`.
	pub id: RubricRef,
	/// Every citation in heading order, with what the run found.
	pub citations: Vec<CitationResult>,
}

impl RubricResult {
	/// The rubric's line in a triage: how many citations are met, failing and
	/// awaiting.
	pub fn row(&self) -> RubricRow {
		let count = |status: fn(&CitationStatus) -> bool| {
			self.citations
				.iter()
				.filter(|citation| status(&citation.status))
				.count() as u32
		};
		RubricRow {
			rubric: self.id.clone(),
			met: count(|status| matches!(status, CitationStatus::Met)),
			failing: count(|status| {
				matches!(status, CitationStatus::Failing { .. })
			}),
			awaiting: count(|status| {
				matches!(status, CitationStatus::Awaiting)
			}),
		}
	}
}

/// One citation as a run reads it.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct CitationResult {
	/// The cited eval.
	pub eval: EvalId,
	/// The level the rubric demands.
	pub level: EvalLevel,
	/// What the run found.
	pub status: CitationStatus,
}

/// Whether a citation is met. A checked eval's citation is met when its check
/// passes, a judged eval's when its grade reaches the cited level.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub enum CitationStatus {
	/// The eval reached the cited level.
	Met,
	/// The eval fell short of it.
	Failing {
		/// The level reached: 0 for a failed check, else the grade's.
		got: EvalLevel,
	},
	/// A judged eval no grader has graded yet.
	Awaiting,
}
