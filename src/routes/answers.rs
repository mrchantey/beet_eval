use super::*;
use crate::prelude::*;
use beet::prelude::*;

/// `eval/answers <package>/<rubric>`: reviews the form's [`AnswerSheet`],
/// `<package>/<rubric>.answers.md` in the build directory, which the builder
/// writes before it edits any of the form's XML. Refused when a section
/// names a cell the blank form lacks or holds no answer; otherwise answers
/// where the sheet stands, each answer under the blank form's text in its
/// cell, the empty cells left unanswered, the edits, and the rubric's
/// structural lines to read the sheet against. `eval/approve` approves it.
#[action(route = "answers/*rubric")]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
pub async fn EvalAnswers(
	/// The reader rubric whose form's answers are reviewed,
	/// `<package>/<rubric>`, the path after `answers`.
	#[param]
	rubric: RubricRef,
	cx: ActionContext<Request>,
) -> Result<Response> {
	let workspace = LoadedWorkspace::of(&cx.caller).await?;
	let checked =
		match CheckedSheet::of(&cx.caller, &workspace, &rubric).await? {
			Ok(checked) => checked,
			Err(refused) => return refused.xok(),
		};
	let state = AnswerState::of(&workspace, &rubric).await?;
	let rubric = workspace
		.rubric(&rubric)
		.ok_or_else(|| bevyhow!("the workspace names no rubric {rubric}"))?;
	Response::ok_text(
		AnswersReview {
			shown: checked.shown,
			state,
			sheet: &checked.sheet,
			cells: &checked.cells,
			rubric,
		}
		.markdown(),
	)
	.xok()
}

/// `eval/approve <package>/<rubric> --by=<who>`: records the form's
/// [`AnswerSheet`] as approved as it stands ([`AnswerApproval`]), refused
/// as `eval/answers` refuses it. Only the person who reviewed the sheet
/// approves it, so no role's toolset carries this verb, and an approval
/// lapses when the sheet changes.
#[action(route = "approve/*rubric")]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
pub async fn EvalApprove(
	/// The reader rubric whose form's answers are approved,
	/// `<package>/<rubric>`, the path after `approve`.
	#[param]
	rubric: RubricRef,
	/// Who approves them, the person who reviewed them.
	#[param]
	by: SmolStr,
	cx: ActionContext<Request>,
) -> Result<Response> {
	let workspace = LoadedWorkspace::of(&cx.caller).await?;
	let checked =
		match CheckedSheet::of(&cx.caller, &workspace, &rubric).await? {
			Ok(checked) => checked,
			Err(refused) => return refused.xok(),
		};
	TableStore::new(workspace.results())
		.table::<AnswerApproval>()
		.push(AnswerApproval {
			rubric,
			by: by.clone(),
			date: Date::today(),
			answers: checked.text,
		})
		.await?;
	Response::ok_text(format!(
		"approved {} by {by}: {} cell(s) and {} edit(s)\n",
		checked.shown,
		checked.sheet.answers.len(),
		checked.sheet.edits.len()
	))
	.xok()
}

/// A form's answer sheet read and checked against its blank form's cells.
struct CheckedSheet {
	/// Its path as a person names it, ie `dist/acme/01-plan.answers.md`.
	shown: String,
	/// Its text.
	text: String,
	/// Its answers.
	sheet: AnswerSheet,
	/// The blank form's cells.
	cells: Vec<FormCell>,
}

impl CheckedSheet {
	/// The sheet of `reference`, or the refusal naming what is wrong with
	/// it: absent, unreadable, or answering what the form lacks.
	async fn of(
		caller: &AsyncEntity,
		workspace: &LoadedWorkspace,
		reference: &RubricRef,
	) -> Result<Result<Self, Response>> {
		workspace.require_clean()?;
		let (package, spec) = FormSource::of(workspace, reference)?;
		let dist = workspace.dist();
		let path = FormBuild::answers_path(reference);
		let shown = format!("{}/{path}", workspace.manifest.dist);
		if !dist.exists(&path).await? {
			return Err(refusal(format!(
				"no answers at {shown} yet: write one `## <cell>` section per cell answered, its cells listed by `eval/cells {reference}`, and an `## Edits` list\n"
			)))
			.xok();
		}
		let text = String::from_utf8(dist.get(&path).await?.to_vec())?;
		let sheet = match AnswerSheet::parse(&text) {
			Ok(sheet) => sheet,
			Err(err) => {
				return Err(refusal(format!("{shown}: {err}\n"))).xok();
			}
		};
		let blank = package.store.blob(spec.form.clone()).get_media().await?;
		let cells = parsed_with(caller, blank, |world, root| {
			FormCell::listing(world, root).xok()
		})
		.await?;
		let problems = sheet.problems(&cells);
		if !problems.is_empty() {
			return Err(refusal(format!(
				"{shown}:\n{}\n",
				problems
					.iter()
					.map(|problem| format!("- {problem}"))
					.collect::<Vec<_>>()
					.join("\n")
			)))
			.xok();
		}
		Ok(Self {
			shown,
			text,
			sheet,
			cells,
		})
		.xok()
	}
}

/// A sheet set out for review beside its form.
struct AnswersReview<'a> {
	shown: String,
	state: AnswerState,
	sheet: &'a AnswerSheet,
	cells: &'a [FormCell],
	rubric: &'a Rubric,
}

impl AnswersReview<'_> {
	/// The review as markdown: each answer under its cell's blank text, the
	/// empty cells unanswered, the edits and the structural lines.
	fn markdown(&self) -> String {
		let mut out = format!(
			"# Answers in {}\n\n{} of {} cell(s) answered and {} edit(s), {}.\n",
			self.shown,
			self.sheet.answers.len(),
			self.cells.len(),
			self.sheet.edits.len(),
			self.state.describe()
		);
		for answer in &self.sheet.answers {
			let blank = self
				.cells
				.iter()
				.find(|cell| cell.cell == answer.cell)
				.map(|cell| cell.text.to_string())
				.filter(|text| !text.is_empty())
				.unwrap_or_else(|| "(empty)".into());
			out.push_str(&format!(
				"\n## {}\n\n> {blank}\n\n{}\n",
				answer.cell, answer.answer
			));
		}
		let unanswered = self
			.cells
			.iter()
			.filter(|cell| {
				cell.text.is_empty()
					&& !self
						.sheet
						.answers
						.iter()
						.any(|answer| answer.cell == cell.cell)
			})
			.map(|cell| format!("`{}`", cell.cell))
			.collect::<Vec<_>>();
		if !unanswered.is_empty() {
			out.push_str(&format!(
				"\n## Empty and unanswered\n\n{}\n",
				unanswered.join(", ")
			));
		}
		out.push_str(&format!("\n## {}\n\n", AnswerSheet::EDITS));
		match self.sheet.edits.is_empty() {
			true => out.push_str("None.\n"),
			false => {
				for edit in &self.sheet.edits {
					out.push_str(&format!("- {edit}\n"));
				}
			}
		}
		out.push_str(
			"\n## Structural lines\n\nRead the answers against each, one at a time.\n\n",
		);
		for (address, line) in self.rubric.structural_lines() {
			out.push_str(&format!("- **{address}** {}\n", line.written()));
		}
		out
	}
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	/// A sheet is refused until its cells are the form's and answered, set
	/// out for review, approved, and unapproved again once it changes.
	#[beet::test]
	async fn reviews_and_approves_answers() {
		let mut fixture = Fixture::new().await;
		let command = "eval/answers acme_course/01-business-plan";
		fixture.refused(command).await.xpect_contains(
			"no answers at dist/acme_course/01-business-plan.answers.md yet",
		);
		fixture
			.write(Fixture::ANSWERS, "## t9/0,0\n\nAcme Stalls\n")
			.await;
		fixture
			.refused(command)
			.await
			.xpect_contains("`t9/0,0` is no cell of the form");
		fixture
			.write(
				Fixture::ANSWERS,
				"## t0/1,0\n\nAcme Stalls\n\n## Edits\n\n- Remove the red sentence.\n",
			)
			.await;
		fixture
			.ok(command)
			.await
			.xpect_contains(
				"1 of 3 cell(s) answered and 1 edit(s), written and awaiting review.",
			)
			.xpect_contains("## t0/1,0\n\n> (empty)\n\nAcme Stalls\n")
			.xpect_contains("- Remove the red sentence.")
			.xpect_contains("## Structural lines");
		fixture
			.refused("eval/approve acme_course/01-business-plan")
			.await
			.xpect_contains("by");
		fixture
			.ok("eval/approve acme_course/01-business-plan --by=ada")
			.await
			.xpect_contains(
				"approved dist/acme_course/01-business-plan.answers.md by ada",
			);
		fixture
			.ok(command)
			.await
			.xpect_contains("approved by ada on ");
		fixture
			.write(Fixture::ANSWERS, "## t0/1,0\n\nAcme Markets\n")
			.await;
		fixture
			.ok(command)
			.await
			.xpect_contains("changed since ada approved it");
	}
}
