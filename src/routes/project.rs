use super::*;
use crate::prelude::*;
use beet::prelude::*;

/// Request params for [`EvalProject`], surfaced in `--help`.
#[derive(Reflect)]
struct ProjectParams {
	/// The rubric, `<package>/<rubric>`, the path after `project`.
	rubric: Option<RubricRef>,
}

/// `eval/project <package>/<rubric>`: the deterministic half of a build, the
/// brief for one form. For every heading of the rubric, the evals it cites,
/// where each is anchored, what is written there today, and the heading's
/// structural lines: the [`ProjectionReport`], written as markdown to
/// `results/projections/<package>-<rubric>.md` and answered, its
/// [`Projection`] for a serde `Accept`. With no rubric named, lists the
/// rubrics.
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
	let listing = || {
		workspace
			.rubrics()
			.iter()
			.map(|(reference, _)| format!("  {reference}\n"))
			.collect::<String>()
	};
	let Some(reference) = params.rubric else {
		return Response::ok_text(format!(
			"name a rubric; the rubrics are:\n{}",
			listing()
		))
		.xok();
	};
	let Some(rubric) = workspace.rubric(&reference) else {
		return refusal(format!(
			"no rubric {reference}; the rubrics are:\n{}",
			listing()
		))
		.xok();
	};
	let mut documents =
		DocumentSet::load(&workspace.docs(), workspace.manifest.docs.clone())
			.await?;
	let projection =
		Projection::new(&workspace, &mut documents, &reference, rubric);
	let path = RelPath::new(format!(
		"projections/{}-{}.md",
		reference.package(),
		reference.rubric()
	));
	let report = || rsx! { <ProjectionReport projection=projection.clone()/> };
	workspace
		.results()
		.insert(&path, markdown_of(&cx.caller, report()).await?)
		.await?;
	DataPage::new(&cx.caller, report(), projection.clone())
		.await?
		.into_response_with_request_parts(
			cx.caller.clone(),
			cx.input.parts().clone(),
		)
		.await
}

/// A rubric read through the anchors, the brief a renderer fills a form
/// from and nothing else.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize)]
pub struct Projection {
	/// The rubric.
	pub rubric: RubricRef,
	/// The day it was projected.
	pub date: Date,
	/// The documents directory its quotes come from.
	pub docs: RelPath,
	/// Every heading citing an eval or carrying a structural line, in order.
	pub headings: Vec<ProjectedHeading>,
}

/// One heading of a [`Projection`].
#[derive(Debug, Clone, PartialEq, Reflect, Serialize)]
pub struct ProjectedHeading {
	/// The heading as the rubric writes it.
	pub title: String,
	/// Its depth below the brief's title, from 2.
	pub depth: usize,
	/// The evals it cites.
	pub citations: Vec<ProjectedCitation>,
	/// Its structural lines, as written with their sources.
	pub structural: Vec<String>,
}

/// One citation of a [`ProjectedHeading`]: the eval, where it is anchored,
/// and what is written there.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize)]
pub struct ProjectedCitation {
	/// The cited eval.
	pub eval: EvalId,
	/// Its anchor, absent for an eval the workspace does not define.
	pub anchor: Option<Address>,
	/// Its statement.
	pub statement: Option<String>,
	/// What is written at the anchor, as markdown.
	pub quote: Option<String>,
}

impl Projection {
	fn new(
		workspace: &LoadedWorkspace,
		documents: &mut DocumentSet,
		reference: &RubricRef,
		rubric: &Rubric,
	) -> Self {
		let mut headings = Vec::new();
		for section in &rubric.sections {
			Self::section(workspace, documents, section, 2, &mut headings);
		}
		Self {
			rubric: reference.clone(),
			date: Date::today(),
			docs: workspace.manifest.docs.clone(),
			headings,
		}
	}

	/// One heading at `depth` and the headings beneath it; a heading citing
	/// nothing and carrying no structural line is left out.
	fn section(
		workspace: &LoadedWorkspace,
		documents: &mut DocumentSet,
		section: &RubricSection,
		depth: usize,
		out: &mut Vec<ProjectedHeading>,
	) {
		if !section.citations.is_empty() || !section.structural.is_empty() {
			out.push(ProjectedHeading {
				title: section.title.to_string(),
				depth,
				citations: section
					.citations
					.iter()
					.map(|citation| {
						let Some(packaged) = workspace.eval(&citation.eval)
						else {
							return ProjectedCitation {
								eval: citation.eval.clone(),
								anchor: None,
								statement: None,
								quote: None,
							};
						};
						let anchor = packaged.eval.anchor_or_namespace();
						ProjectedCitation {
							eval: citation.eval.clone(),
							quote: documents
								.text_at(&anchor)
								.filter(|text| !text.is_empty()),
							anchor: Some(anchor),
							statement: Some(
								packaged.eval.statement.to_string(),
							),
						}
					})
					.collect(),
				structural: section
					.structural
					.iter()
					.map(StructuralLine::written)
					.collect(),
			});
		}
		for child in &section.sections {
			Self::section(workspace, documents, child, depth + 1, out);
		}
	}
}

/// The brief a renderer fills a form from: for each heading of the rubric,
/// the evals it cites, where each is anchored, what is written there, and
/// the heading's structural lines.
#[template]
pub fn ProjectionReport(
	#[prop(required)] projection: Projection,
) -> impl Bundle {
	let reference = projection.rubric.to_string();
	let headings = projection
		.headings
		.iter()
		.map(|heading| {
			let title = Element::new(format!("h{}", heading.depth.min(6)))
				.with_inner_text(&heading.title);
			let citations = (!heading.citations.is_empty()).then(|| {
				let items = heading
					.citations
					.iter()
					.map(|citation| {
						let said = match (&citation.anchor, &citation.statement) {
							(Some(anchor), Some(statement)) => rsx! {
								" at "<code>{anchor.to_string()}</code>{format!(": {statement}")}
							}
							.any_bundle(),
							_ => rsx! { ": an eval the workspace does not define" }.any_bundle(),
						};
						let quote = match (&citation.anchor, &citation.quote) {
							(_, Some(quote)) => {
								let lines = quote
									.lines()
									.filter(|line| !line.trim().is_empty())
									.map(|line| rsx! { <p>{line.to_string()}</p> })
									.collect::<Vec<_>>();
								rsx! { <blockquote>{lines}</blockquote> }.any_bundle()
							}
							(Some(anchor), None) => rsx! {
								<blockquote><p>{format!("(nothing written at {anchor} yet)")}</p></blockquote>
							}
							.any_bundle(),
							(None, None) => rsx! { <></> }.any_bundle(),
						};
						rsx! {
							<p><code>{citation.eval.to_string()}</code>{said}</p>
							{quote}
						}
					})
					.collect::<Vec<_>>();
				rsx! { <p>"Draws on:"</p>{items} }
			});
			let structural = (!heading.structural.is_empty()).then(|| {
				let items = heading
					.structural
					.iter()
					.map(|line| rsx! { <li>{line.clone()}</li> })
					.collect::<Vec<_>>();
				rsx! { <p>"Structural:"</p><ol>{items}</ol> }
			});
			rsx! { <>{title}{citations}{structural}</> }
		})
		.collect::<Vec<_>>();
	rsx! {
		<h1>{format!("Projection: {reference}")}</h1>
		<p>
			"Written by "<code>{format!("eval/project {reference}")}</code>
			{format!(" on {}: for each heading of the rubric ", projection.date)}
			<code>{reference.clone()}</code>
			", the evals it cites, where each is anchored in "
			<code>{format!("{}/", projection.docs)}</code>
			", what is written there, and the heading's structural lines. A renderer fills the form from this and nothing else; a structural line is checked on the result."
		</p>
		{headings}
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
			.xpect_starts_with("# Projection: acme_course/01-business-plan\n");
		fixture
			.read("results/projections/acme_course-01-business-plan.md")
			.await
			.xpect_contains("## Whole document\n\nStructural:\n\n1. Every red instruction sentence is deleted. [form]\n")
			.xpect_contains(
				"## 1.2 Business Description\n\nDraws on:\n\n`product.origin` at `product#idea`: Where the idea came from",
			)
			.xpect_contains("produced it.\n\n> The idea came from our own weekend stall.");
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
