use crate::prelude::*;
use beet::prelude::*;

/// A person's approval of a form's answers: a row of the results'
/// `approvals` table, keyed on the rubric, holding the [`AnswerSheet`]'s text
/// as approved, so a sheet changed since reads as unapproved and the
/// approved answers are kept whatever becomes of the build directory.
///
/// Written only through `eval/answers --approve`, by the owner who reviewed
/// the sheet, never by the builder that wrote it.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct AnswerApproval {
	/// The rubric whose form's answers are approved.
	pub rubric: RubricRef,
	/// Who approved them.
	pub by: SmolStr,
	/// The day they were approved.
	pub date: Date,
	/// The sheet's text as approved.
	pub answers: String,
}

/// A row of the results' `approvals` table, keyed on its rubric.
impl TableStoreRow for AnswerApproval {
	fn table_name() -> SmolStr { "approvals".into() }
	fn key(&self) -> TableKey { self.rubric.as_str().into() }
}

/// Where a form's answers stand.
#[derive(Debug, Clone, PartialEq)]
pub enum AnswerState {
	/// No answer sheet is written.
	Unwritten,
	/// A sheet is written and nobody has approved it.
	Awaiting,
	/// The sheet has changed since it was approved.
	Changed(AnswerApproval),
	/// The sheet is approved as it stands.
	Approved(AnswerApproval),
}

impl AnswerState {
	/// Where the answers of `rubric` stand in `workspace`.
	pub async fn of(
		workspace: &LoadedWorkspace,
		rubric: &RubricRef,
	) -> Result<Self> {
		let path = FormBuild::answers_path(rubric);
		let dist = workspace.dist();
		if !dist.exists(&path).await? {
			return Self::Unwritten.xok();
		}
		let text = String::from_utf8(dist.get(&path).await?.to_vec())?;
		let approvals =
			TableStore::new(workspace.results()).table::<AnswerApproval>();
		match approvals.exists(rubric.as_str()).await? {
			false => Self::Awaiting,
			true => {
				let approval = approvals.get(rubric.as_str()).await?;
				match approval.answers == text {
					true => Self::Approved(approval),
					false => Self::Changed(approval),
				}
			}
		}
		.xok()
	}

	/// The state in a sentence, ie `approved by ada on 2026-11-22`.
	pub fn describe(&self) -> String {
		match self {
			Self::Unwritten => "no answers are written".into(),
			Self::Awaiting => "written and awaiting review".into(),
			Self::Changed(approval) => format!(
				"changed since {} approved it on {}, so awaiting review",
				approval.by, approval.date
			),
			Self::Approved(approval) => {
				format!("approved by {} on {}", approval.by, approval.date)
			}
		}
	}
}

/// How far a reader rubric's form has come, as `eval/next` weighs it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormStage {
	/// No answers are written for it.
	Unanswered,
	/// Its answers await a person's review, written or changed since one.
	Awaiting,
	/// Its answers are approved and it is not built.
	Approved,
	/// Its built form has its cells listed in the build directory.
	Built,
}

impl FormStage {
	/// The stage of the form of `rubric` in `workspace`.
	pub async fn of(
		workspace: &LoadedWorkspace,
		rubric: &RubricRef,
	) -> Result<Self> {
		if workspace
			.dist()
			.exists(&FormBuild::cells_path(rubric))
			.await?
		{
			return Self::Built.xok();
		}
		match AnswerState::of(workspace, rubric).await? {
			AnswerState::Unwritten => Self::Unanswered,
			AnswerState::Awaiting | AnswerState::Changed(_) => Self::Awaiting,
			AnswerState::Approved(_) => Self::Approved,
		}
		.xok()
	}
}
