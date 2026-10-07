use crate::prelude::*;
use beet::prelude::*;

/// One run over the subject, the computation `eval/results` writes and
/// `eval/next` triages from: the workspace read clean, its documents, every
/// check route's verdict and the [`Results`] every rubric reads.
pub(crate) struct Run {
	/// The workspace.
	pub workspace: LoadedWorkspace,
	/// Its documents.
	pub documents: DocumentSet,
	/// Every checked eval's verdict, by id.
	pub verdicts: Vec<(EvalId, CheckVerdict)>,
	/// What the run found.
	pub results: Results,
}

impl Run {
	/// Runs the workspace the verb on `caller` resolves: refused when the
	/// workspace could not be read whole, every check called through its
	/// route, the grade rows merged.
	pub async fn of(caller: &AsyncEntity) -> Result<Self> {
		let workspace = LoadedWorkspace::of(caller).await?;
		workspace.require_clean()?;
		let documents = DocumentSet::load(
			&workspace.docs(),
			workspace.manifest.docs.clone(),
		)
		.await?;
		let mut verdicts = Vec::new();
		for packaged in &workspace.evals {
			let Some(check) = packaged.eval.levels.check() else {
				continue;
			};
			let verdict = check.call(caller).await.unwrap_or_else(|err| {
				CheckVerdict::fail(
					format!("check/{} could not run: {err}", check.route),
					Vec::<SmolStr>::new(),
				)
			});
			verdicts.push((packaged.eval.id.clone(), verdict));
		}
		let (grades, unreadable) =
			json_ext::rows::<Grade>(&workspace.results()).await?;
		let mut results = Results::compute(
			Date::today(),
			&workspace.evals,
			&verdicts,
			grades,
			&workspace.rubrics(),
		);
		if let Some(grades) = results.grades.as_mut() {
			grades.problems.extend(unreadable);
		}
		Self {
			workspace,
			documents,
			verdicts,
			results,
		}
		.xok()
	}
}
