use super::*;
use crate::prelude::*;
use beet::prelude::*;

/// Request params for [`EvalProject`], surfaced in `--help`.
#[derive(Reflect)]
struct ProjectParams {
	/// The rubric, `<package>/<rubric>`, the path after `project`.
	rubric: Vec<String>,
}

/// `eval/project <package>/<rubric>`: the deterministic half of a build, the
/// brief for one form. For every heading of the rubric, the evals it cites,
/// where each is anchored, what is written there today, and the heading's
/// structural lines, written to `results/projections/<package>-<rubric>.md`.
/// With no rubric named, lists the rubrics.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
#[require(
	PathPartial = PathPartial::new("project/*rubric?"),
	ParamsPartial = ParamsPartial::new::<ProjectParams>()
)]
pub async fn EvalProject(cx: ActionContext<Request>) -> Result<Response> {
	let params = cx.input.parse_params::<ProjectParams>()?;
	let workspace = LoadedWorkspace::of(&cx.caller).await?;
	workspace.require_clean()?;
	let target = params.rubric.join("/");
	let listing = || {
		workspace
			.rubrics()
			.iter()
			.map(|(reference, _)| format!("  {reference}\n"))
			.collect::<String>()
	};
	if target.is_empty() {
		return Response::ok_text(format!(
			"name a rubric; the rubrics are:\n{}",
			listing()
		))
		.xok();
	}
	let Some((reference, rubric)) =
		RubricRef::parse(&target).ok().and_then(|reference| {
			workspace
				.rubric(&reference)
				.map(|rubric| (reference, rubric))
		})
	else {
		return refusal(format!(
			"no rubric {target}; the rubrics are:\n{}",
			listing()
		))
		.xok();
	};
	let documents =
		DocumentSet::load(&workspace.docs(), workspace.manifest.docs.clone())
			.await?;
	let brief =
		Projection::new(&workspace, &documents, &reference).write(rubric);
	let path = RelPath::new(format!(
		"projections/{}-{}.md",
		reference.package(),
		reference.rubric()
	));
	workspace.results().insert(&path, brief.text).await?;
	Response::ok_text(format!(
		"{}/{path} written: {} headings\n",
		workspace.manifest.results, brief.headings
	))
	.xok()
}

/// A rubric read through the anchors.
struct Projection<'a> {
	workspace: &'a LoadedWorkspace,
	documents: &'a DocumentSet,
	reference: &'a RubricRef,
}

/// A written brief and how many headings it carries.
struct Brief {
	text: String,
	headings: usize,
}

impl<'a> Projection<'a> {
	fn new(
		workspace: &'a LoadedWorkspace,
		documents: &'a DocumentSet,
		reference: &'a RubricRef,
	) -> Self {
		Self {
			workspace,
			documents,
			reference,
		}
	}

	fn write(&self, rubric: &Rubric) -> Brief {
		let mut out = vec![
			format!("# Projection: {}", self.reference),
			String::new(),
			format!(
				"Written by `eval/project {reference}` on {}: for each heading of the \
				 rubric `{reference}`, the evals it cites, where each is anchored in \
				 `{}/`, what is written there, and the heading's structural lines. A \
				 renderer fills the form from this and nothing else; a structural \
				 line is checked on the result.",
				Date::today(),
				self.workspace.manifest.docs,
				reference = self.reference,
			),
			String::new(),
		];
		let mut headings = 0;
		for section in &rubric.sections {
			self.section(section, 2, &mut out, &mut headings);
		}
		Brief {
			text: format!("{}\n", out.join("\n").trim_end()),
			headings,
		}
	}

	/// One heading at `depth` hashes and the headings beneath it; a heading
	/// citing nothing and carrying no structural line is left out.
	fn section(
		&self,
		section: &RubricSection,
		depth: usize,
		out: &mut Vec<String>,
		headings: &mut usize,
	) {
		if !section.citations.is_empty() || !section.structural.is_empty() {
			*headings += 1;
			out.extend([
				format!("{} {}", "#".repeat(depth), section.title),
				String::new(),
			]);
			if !section.citations.is_empty() {
				out.extend(["Draws on:".to_string(), String::new()]);
				for citation in &section.citations {
					self.citation(citation, out);
				}
				out.push(String::new());
			}
			if !section.structural.is_empty() {
				out.extend(["Structural:".to_string(), String::new()]);
				for (index, line) in section.structural.iter().enumerate() {
					out.push(format!("{}. {}", index + 1, line.written()));
				}
				out.push(String::new());
			}
		}
		for child in &section.sections {
			self.section(child, depth + 1, out, headings);
		}
	}

	/// A cited eval with the text at its anchor quoted.
	fn citation(&self, citation: &Citation, out: &mut Vec<String>) {
		let Some(packaged) = self.workspace.eval(&citation.eval) else {
			out.push(format!(
				"- `{}`: an eval the workspace does not define",
				citation.eval
			));
			return;
		};
		let anchor = packaged.eval.anchor_or_namespace();
		out.push(format!(
			"- `{}` at `{anchor}`: {}",
			citation.eval, packaged.eval.statement
		));
		match self
			.documents
			.text_at(&anchor)
			.filter(|text| !text.is_empty())
		{
			Some(text) => {
				out.extend(text.split('\n').map(|line| format!("  > {line}")))
			}
			None => out.push(format!("  > (nothing written at {anchor} yet)")),
		}
	}
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	#[beet::test]
	async fn writes_the_brief() {
		let mut fixture = Fixture::new().await;
		fixture
			.ok("eval/project acme_course/01-business-plan")
			.await
			.xpect_eq(
				"results/projections/acme_course-01-business-plan.md written: 2 headings\n",
			);
		fixture
			.read("results/projections/acme_course-01-business-plan.md")
			.await
			.xpect_contains("## Whole document\n\nStructural:\n\n1. Every red instruction sentence is deleted. [form]\n")
			.xpect_contains(
				"## 1.2 Business Description\n\nDraws on:\n\n- `product.origin` at `product#idea`: Where the idea came from",
			)
			.xpect_contains("  > The idea came from our own weekend stall.");
		fixture
			.ok("eval/project")
			.await
			.xpect_eq("name a rubric; the rubrics are:\n  acme_biz/owner\n  acme_course/01-business-plan\n");
		fixture
			.refused("eval/project acme_course/nothing")
			.await
			.xpect_contains("no rubric");
	}
}
