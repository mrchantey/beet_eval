use super::*;
use crate::prelude::*;
use beet::prelude::*;

/// `eval/build <suite>/<rubric>`: refused until the form's
/// [`AnswerSheet`] is approved as it stands (`eval/answers`), so nothing is
/// written into the form a person has not reviewed; then unpacks the
/// reader's blank form named by
/// the rubric's render spec into `<suite>/<rubric>/` of the build
/// directory when it is not yet there, or afresh with `--fresh`, zips those
/// parts, as the builder has edited them, into the built form beside them,
/// and writes its [`CellsReport`], to read against the rubric's structural
/// lines. Parts that no longer read as their format refuse the build.
#[action(route = "build/*rubric")]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
pub async fn EvalBuild(
	/// The reader rubric whose form is built, `<suite>/<rubric>`, the path after `build`.
	#[param]
	rubric: RubricRef,
	/// Unpacks the blank form again, discarding every edit to its parts.
	#[param]
	fresh: bool,
	cx: ActionContext<Request>,
) -> Result<Response> {
	let workspace = LoadedWorkspace::of(&cx.caller).await?;
	workspace.require_clean()?;
	let reference = rubric;
	let (suite, spec) = FormSource::of(&workspace, &reference)?;
	let state = AnswerState::of(&workspace, &reference).await?;
	if !matches!(state, AnswerState::Approved(_)) {
		return refusal(format!(
			"{reference} is not built while its answers are not approved: {}; write them in {}/{} and review them with `eval/answers {reference}`\n",
			state.describe(),
			workspace.manifest.dist,
			FormBuild::answers_path(&reference)
		))
		.xok();
	}
	let dist = workspace.dist();
	let dist_dir = &workspace.manifest.dist;
	let output = RelPath::new(reference.suite()).join(spec.output.as_str());
	let parts_dir = FormBuild::parts_dir(&reference);
	let parts = dist.with_subdir(parts_dir.clone());
	let blank = suite.store.blob(spec.form.clone()).get_media().await?;
	let mut log = Vec::new();
	if fresh || !FormBuild::is_unpacked(&parts).await? {
		let count = FormBuild::unpack(&parts, &blank).await?;
		log.push(format!(
			"unpacked the blank form's {count} parts into {dist_dir}/{parts_dir}/"
		));
	}
	let built = FormBuild::pack(&parts, blank.media_type().clone()).await?;
	let cells = parsed_with(&cx.caller, built.clone(), |world, root| {
		FormCell::listing(world, root).xok()
	})
	.await
	.map_err(|err| {
		bevyhow!("the parts in {dist_dir}/{parts_dir}/ no longer read: {err}")
	})?;
	dist.insert(&output, built.bytes().to_vec()).await?;
	let cells_path = FormBuild::cells_path(&reference);
	let count = cells.len();
	let report = rsx! {
		<CellsReport
			output={format!("{dist_dir}/{output}")}
			rubric=reference.clone()
			date=Date::today()
			cells=cells
		/>
	};
	dist.insert(&cells_path, markdown_of(&cx.caller, report, None).await?)
		.await?;
	log.push(format!(
		"{dist_dir}/{output} written; {count} cells listed in {dist_dir}/{cells_path}"
	));
	Response::ok_text(format!("{}\n", log.join("\n"))).xok()
}

/// Where a rubric's blank form comes from.
pub(crate) struct FormSource;

impl FormSource {
	/// The reader suite and render spec of `reference`.
	pub fn of<'a>(
		workspace: &'a LoadedWorkspace,
		reference: &RubricRef,
	) -> Result<(&'a LoadedSuite, &'a RenderSpec)> {
		let suite = workspace.suite(reference.suite()).ok_or_else(|| {
			bevyhow!("the workspace names no suite {}", reference.suite())
		})?;
		let spec = suite.render_spec(reference.rubric()).ok_or_else(|| {
			bevyhow!(
				"{} has no render spec for {}, so its form is not built",
				reference.suite(),
				reference.rubric()
			)
		})?;
		(suite, spec).xok()
	}
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	/// A form whose answers are not approved is refused; the first build
	/// unpacks the blank form and lists its cells; an edit to its parts
	/// reaches the next build, `--fresh` discards it, and parts that no
	/// longer read refuse the build.
	#[beet::test]
	async fn builds_the_form_from_its_parts() {
		let mut fixture = Fixture::new().await;
		let build = "eval/build acme_course/01-business-plan";
		fixture.refused(build).await.xpect_contains(
			"not built while its answers are not approved: no answers are written",
		);
		fixture.approved_answers("## t0/1,0\n\nAcme Stalls\n").await;
		let written = "dist/acme_course/01-business-plan.docx written; 3 cells listed in dist/acme_course/01-business-plan.cells.md\n";
		fixture.ok(build).await.xpect_eq(format!(
			"unpacked the blank form's 3 parts into dist/acme_course/01-business-plan/\n{written}"
		));
		let document = "dist/acme_course/01-business-plan/word/document.xml";
		let xml = fixture.read(document).await;
		fixture
			.store
			.insert(
				&RelPath::new(document),
				xml.replacen(
					"<w:tc><w:p/></w:tc>",
					"<w:tc><w:p><w:r><w:t>Acme Stalls</w:t></w:r></w:p></w:tc>",
					1,
				),
			)
			.await
			.unwrap();
		fixture.ok(build).await.xpect_eq(written);
		fixture
			.read("dist/acme_course/01-business-plan.cells.md")
			.await
			.xpect_contains("| Cell | Text |\n|---|---|\n")
			.xpect_contains("| t0/1,0 | Acme Stalls |\n");
		// a built form takes it out of the triage's build queue
		fixture
			.ok("eval/next")
			.await
			.xpect_starts_with("## Next: write product");

		fixture
			.ok(&format!("{build} --fresh"))
			.await
			.xpect_starts_with("unpacked");
		fixture
			.read("dist/acme_course/01-business-plan.cells.md")
			.await
			.xpect_contains("| t0/1,0 | (empty) |\n");

		fixture
			.store
			.insert(&RelPath::new(document), "<w:document>")
			.await
			.unwrap();
		fixture
			.refused(build)
			.await
			.xpect_contains("no longer read");
	}
}
