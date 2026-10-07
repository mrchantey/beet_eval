use crate::prelude::*;
use beet::prelude::*;

/// Request params for [`EvalBuild`], surfaced in `--help`.
#[derive(Reflect)]
struct BuildParams {
	/// The reader rubric whose form is built, `<package>/<rubric>`, the path
	/// after `build`.
	rubric: RubricRef,
}

/// `eval/build <package>/<rubric>`: copies the reader's blank form named by
/// the rubric's render spec into the build directory, applies the fill spec
/// at `dist/<package>/<rubric>.fill.json` when there is one, and dumps the
/// result's cells beside it, to read against the rubric's structural lines.
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
	let blank = package.store.get(&spec.form).await?;
	let mut log = Vec::new();
	let cells = match FormKind::of(&spec.form)? {
		FormKind::Word => {
			let mut form = WordDocument::from_bytes(blank.to_vec())?;
			match &fill {
				Some(fill) => log.extend(fill.apply_word(&mut form)?),
				None => log.push(FormSource::unfilled(&reference)),
			}
			dist.insert(&output, form.to_bytes()?).await?;
			CellsDump::word(&form)
		}
		FormKind::Workbook => {
			let mut form = Workbook::from_bytes(blank.to_vec())?;
			match &fill {
				Some(fill) => log.extend(fill.apply_workbook(&mut form)?),
				None => log.push(FormSource::unfilled(&reference)),
			}
			dist.insert(&output, form.to_bytes()?).await?;
			CellsDump::workbook(&form)?
		}
	};
	let dump_path = CellsDump::path(&reference);
	let dist_dir = &workspace.manifest.dist;
	dist.insert(
		&dump_path,
		cells.to_markdown(&format!("{dist_dir}/{output}"), &reference),
	)
	.await?;
	log.push(format!(
		"{dist_dir}/{output} written; {} cells dumped to {dist_dir}/{dump_path}",
		cells.len()
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
				 dist/acme_course/01-business-plan.docx written; 3 cells dumped to dist/acme_course/01-business-plan.cells.md\n",
			);
		fixture
			.read("dist/acme_course/01-business-plan.cells.md")
			.await
			.xpect_contains("| t1r1c2 | Acme Stalls |\n| t2r1c1 | Briefly describe your business: / Acme Stalls rents fitted market stalls. |\n");
		// a built form takes it out of the triage's build queue
		fixture
			.ok("eval/next")
			.await
			.xpect_starts_with("## Next: write product");
	}
}
