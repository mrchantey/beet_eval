use crate::prelude::*;
use beet::prelude::*;

/// Request params for [`EvalBuild`], surfaced in `--help`.
#[derive(Reflect)]
struct BuildParams {
	/// The reader rubric whose form is built, `<package>/<rubric>`, the path
	/// after `build`.
	rubric: RubricRef,
}

/// `eval/build <package>/<rubric>`: parses the reader's blank form named by
/// the rubric's render spec, a Word file, a workbook or any document with
/// tables, applies the fill spec at `dist/<package>/<rubric>.fill.json` when
/// there is one, renders the result in the form's own format into the build
/// directory, and writes its [`CellsReport`] beside it, to read against the
/// rubric's structural lines.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
#[require(
	PathPartial = PathPartial::new("build/*rubric"),
	ParamsPartial = ParamsPartial::new::<BuildParams>()
)]
pub async fn EvalBuild(cx: ActionContext<Request>) -> Result<Response> {
	let params = cx.input.parse_params::<BuildParams>()?;
	let workspace = LoadedWorkspace::of(&cx.caller).await?;
	workspace.require_clean()?;
	let reference = params.rubric;
	let (package, spec) = FormSource::of(&workspace, &reference)?;
	let dist = workspace.dist();
	let output = RelPath::new(reference.package()).join(spec.output.as_str());
	let fill =
		json_ext::read_optional::<FillSpec>(&dist, &FillSpec::path(&reference))
			.await?;
	let blank = package.store.blob(spec.form.clone()).get_media().await?;
	let unfilled = FormSource::unfilled(&reference);
	let (mut log, built, cells) =
		FormSource::read(&cx.caller, blank, move |world, root| {
			let log = match &fill {
				Some(fill) => fill.apply(world, root)?,
				None => vec![unfilled],
			};
			let media_type = world
				.entity(root)
				.get::<OoxmlPackage>()
				.map(|package| package.media_type().clone())
				.unwrap_or(MediaType::Markdown);
			let built = MediaRenderer::default().render(
				&mut RenderContext::new(root, world)
					.with_accepts(vec![media_type]),
			)?;
			let cells =
				world.with_state::<TableCells, _>(|cells| cells.listing(root));
			(log, built, cells).xok()
		})
		.await?;
	dist.insert(&output, built.bytes().to_vec()).await?;
	let dist_dir = &workspace.manifest.dist;
	let cells_path = FillSpec::cells_path(&reference);
	let count = cells.len();
	let report = rsx! {
		<CellsReport
			output={format!("{dist_dir}/{output}")}
			rubric=reference.clone()
			date=Date::today()
			cells=cells
		/>
	};
	dist.insert(&cells_path, markdown_of(&cx.caller, report).await?)
		.await?;
	log.push(format!(
		"{dist_dir}/{output} written; {count} cells listed in {dist_dir}/{cells_path}"
	));
	Response::ok_text(format!("{}\n", log.join("\n"))).xok()
}

/// Where a rubric's blank form comes from.
pub(crate) struct FormSource;

impl FormSource {
	/// The reader package and render spec of `reference`.
	pub fn of<'a>(
		workspace: &'a LoadedWorkspace,
		reference: &RubricRef,
	) -> Result<(&'a LoadedPackage, &'a RenderSpec)> {
		let package =
			workspace.package(reference.package()).ok_or_else(|| {
				bevyhow!(
					"the workspace names no package {}",
					reference.package()
				)
			})?;
		let spec =
			package.render_spec(reference.rubric()).ok_or_else(|| {
				bevyhow!(
					"{} has no render spec for {}, so its form is not built",
					reference.package(),
					reference.rubric()
				)
			})?;
		(package, spec).xok()
	}

	fn unfilled(reference: &RubricRef) -> String {
		format!(
			"no fill spec at dist/{}; copied the blank form unfilled",
			FillSpec::path(reference)
		)
	}

	/// Parses a form into the caller's world and runs `func` on its root,
	/// which is despawned after, whatever `func` answers.
	pub async fn read<O: 'static + Send + Sync>(
		caller: &AsyncEntity,
		form: MediaBytes,
		func: impl 'static + Send + FnOnce(&mut World, Entity) -> Result<O>,
	) -> Result<O> {
		caller
			.world()
			.with(move |world: &mut World| -> Result<O> {
				let root = world.spawn_empty().id();
				let read = MediaParser::new()
					.parse(ParseContext::new(
						&mut world.entity_mut(root),
						&form,
					))
					.map_err(BevyError::from)
					.and_then(|_| func(world, root));
				world.entity_mut(root).despawn();
				read
			})
			.await
	}
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	#[beet::test]
	async fn fills_the_form_and_dumps_its_cells() {
		let mut fixture = Fixture::new().await;
		let spec = FillSpec {
			ops: vec![
				FillOp::Set {
					cell: CellRef::parse("t1r1c2").unwrap(),
					text: "Acme Stalls".into(),
				},
				FillOp::Append {
					cell: CellRef::parse("t2r1c1").unwrap(),
					text: "Acme Stalls rents fitted market stalls.".into(),
				},
				FillOp::Delete {
					text: "Please delete this sentence".into(),
				},
				FillOp::Check {
					label: "Surveys".into(),
				},
				FillOp::Check {
					label: "Focus groups".into(),
				},
			],
		};
		json_ext::write(
			&fixture.store.with_subdir(RelPath::new("dist")),
			&RelPath::new("acme_course/01-business-plan.fill.json"),
			&spec,
		)
		.await
		.unwrap();
		fixture
			.ok("eval/build acme_course/01-business-plan")
			.await
			.xpect_eq(
				"set t1r1c2\nappend t2r1c1\ndelete \"Please delete this sentence\": 1 paragraph(s)\n\
				 check \"Surveys\": 1\ncheck \"Focus groups\": 0\n  WARNING op 5: no checkbox carries that label\n\
				 dist/acme_course/01-business-plan.docx written; 3 cells listed in dist/acme_course/01-business-plan.cells.md\n",
			);
		fixture
			.read("dist/acme_course/01-business-plan.cells.md")
			.await
			.xpect_contains("| Cell | Text |\n|---|---|\n")
			.xpect_contains("| t1r1c2 | Acme Stalls |\n| t2r1c1 | Briefly describe your business: / Acme Stalls rents fitted market stalls. |\n");
		// a built form takes it out of the triage's build queue
		fixture
			.ok("eval/next")
			.await
			.xpect_starts_with("## Next: write product");
	}
}
