use crate::prelude::*;
use beet::prelude::*;

/// Request params for [`EvalWorksheet`], surfaced in `--help`.
#[derive(Reflect)]
struct WorksheetParams {
	/// One document's worksheet, the `:document` path segment.
	document: Option<SmolStr>,
}

/// `eval/worksheet [<document>]`: every judged eval anchored in each
/// document, in the outline's order and then by section, with its statement
/// and level lines: what a grader grades and a writer writes towards. The
/// [`WorksheetReport`] for a markup `Accept`, the documents' evals for a
/// serde one.
#[action(route = "worksheet/:document?")]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
#[require(ParamsPartial = ParamsPartial::new::<WorksheetParams>())]
pub async fn EvalWorksheet(
	cx: ActionContext<Request>,
) -> Result<DataPage<Vec<WorksheetDocument>>> {
	let params = cx.input.parse_params::<WorksheetParams>()?;
	let workspace = LoadedWorkspace::of(&cx.caller).await?;
	workspace.require_clean()?;
	let outline = workspace.outline()?;
	// every judged eval by the address it is anchored at
	let mut by_anchor = Vec::<(Address, Vec<&Eval>)>::new();
	for packaged in workspace
		.evals
		.iter()
		.filter(|packaged| !packaged.eval.is_checked())
	{
		let anchor = packaged.eval.anchor_or_namespace();
		match by_anchor
			.iter_mut()
			.find(|(existing, _)| *existing == anchor)
		{
			Some((_, evals)) => evals.push(&packaged.eval),
			None => by_anchor.push((anchor, vec![&packaged.eval])),
		}
	}
	let mut order = outline
		.documents
		.iter()
		.map(|document| SmolStr::new(&document.name))
		.collect::<Vec<_>>();
	for (anchor, _) in &by_anchor {
		let document = SmolStr::new(anchor.document_name());
		if !order.contains(&document) {
			order.push(document);
		}
	}
	let names = match params.document {
		Some(document) => vec![document],
		None => order,
	};
	let anchored = |address: &str| {
		by_anchor
			.iter()
			.find(|(anchor, _)| anchor.as_str() == address)
			.map(|(_, evals)| {
				evals.iter().map(|eval| (*eval).clone()).collect()
			})
			.unwrap_or_default()
	};
	let documents = names
		.iter()
		.map(|document| WorksheetDocument {
			document: document.clone(),
			evals: anchored(document),
			sections: outline
				.documents
				.iter()
				.find(|spec| spec.name == document.as_str())
				.map(|spec| spec.sections.as_slice())
				.unwrap_or_default()
				.iter()
				.map(|section| WorksheetSection {
					heading: section.heading.clone(),
					evals: anchored(&format!(
						"{document}#{}",
						Address::slug(&section.heading)
					)),
				})
				.filter(|section| !section.evals.is_empty())
				.collect(),
		})
		.collect::<Vec<_>>();
	let report = rsx! { <WorksheetReport documents=documents.clone()/> };
	DataPage::new(&cx.caller, report, documents).await
}

/// One document of a worksheet: the judged evals anchored at the whole
/// document, then those of each of its sections.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct WorksheetDocument {
	/// The document's name.
	pub document: SmolStr,
	/// The evals anchored at the whole document.
	pub evals: Vec<Eval>,
	/// Each section with an eval anchored there, in the outline's order.
	pub sections: Vec<WorksheetSection>,
}

/// One section of a worksheet and the judged evals anchored there.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct WorksheetSection {
	/// The heading as the outline writes it.
	pub heading: SmolStr,
	/// The evals, by id.
	pub evals: Vec<Eval>,
}

/// The worksheet as a grader reads it: each document a heading, each section
/// beneath, each eval its id, its kind of levels and its statement, with its
/// own level lines and note beneath it.
#[template]
pub fn WorksheetReport(documents: Vec<WorksheetDocument>) -> impl Bundle {
	let entries = |evals: &[Eval]| {
		let items = evals
			.iter()
			.map(|eval| {
				let mut lines = Vec::new();
				if let Levels::Custom { l0, l1, l2, l3 } = &eval.levels {
					for (level, line) in
						[l0, l1, l2, l3].into_iter().enumerate()
					{
						lines.push(
							rsx! { <li>{format!("{level}: {line}")}</li> }
								.any_bundle(),
						);
					}
				}
				if let Some(note) = &eval.note {
					lines.push(
						rsx! { <li>{format!("Note: {note}")}</li> }
							.any_bundle(),
					);
				}
				let lines =
					(!lines.is_empty()).then(|| rsx! { <ul>{lines}</ul> });
				rsx! {
					<li>
						<strong>{eval.id.to_string()}</strong>
						{format!(" ({}): {}", eval.levels.word(), eval.statement)}
						{lines}
					</li>
				}
			})
			.collect::<Vec<_>>();
		(!items.is_empty()).then(|| rsx! { <ul>{items}</ul> })
	};
	let documents = documents
		.iter()
		.map(|document| {
			let sections = document
				.sections
				.iter()
				.map(|section| {
					rsx! {
						<h3>{section.heading.to_string()}</h3>
						{entries(&section.evals)}
					}
				})
				.collect::<Vec<_>>();
			rsx! {
				<h2>{document.document.to_string()}</h2>
				{entries(&document.evals)}
				{sections}
			}
		})
		.collect::<Vec<_>>();
	rsx! { <>{documents}</> }
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	#[beet::test]
	async fn lists_judged_evals_by_section() {
		Fixture::new()
			.await
			.ok("eval/worksheet product")
			.await
			.xpect_eq(
				"## product\n\n\
				 ### Idea\n\n\
				 - **product.alternatives-considered** (custom): Other ideas were considered and the chosen idea is preferred to them for a reason.\n\
				 \x20 - 0: No alternative is named.\n\
				 \x20 - 1: Alternatives are named without a reason for passing them over.\n\
				 \x20 - 2: Each alternative carries why the chosen idea beat it.\n\
				 \x20 - 3: The alternatives were weighed on the same grounds as the chosen idea, and the comparison is written down.\n\
				 - **product.origin** (generic): Where the idea came from is told: the experience or the demand that produced it.\n\n\
				 ### Pricing\n\n\
				 - **product.pricing-justified** (generic): Each price is justified by what it costs to deliver and what customers pay elsewhere.\n\
				 - **product.quote-held** (binary): A supplier's quote is held for every kit the business rents.\n\
				 \x20 - Note: A judged binary eval: met or not, with no check to decide it.\n",
			);
	}
}
