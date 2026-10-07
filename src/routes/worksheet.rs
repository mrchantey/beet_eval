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
/// and level lines: what a grader grades and a writer writes towards.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
#[require(
	PathPartial = PathPartial::new("worksheet/:document?"),
	ParamsPartial = ParamsPartial::new::<WorksheetParams>()
)]
pub async fn EvalWorksheet(cx: ActionContext<Request>) -> Result<Response> {
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
	let documents = match params.document {
		Some(document) => vec![document],
		None => order,
	};
	let anchored = |address: &str| {
		by_anchor
			.iter()
			.find(|(anchor, _)| anchor.as_str() == address)
			.map(|(_, evals)| evals.as_slice())
			.unwrap_or_default()
	};
	let mut out = String::new();
	for document in &documents {
		out.push_str(&format!("## {document}\n\n"));
		let direct = anchored(document);
		if !direct.is_empty() {
			for eval in direct {
				out.push_str(&Worksheet::entry(eval));
			}
			out.push('\n');
		}
		let headings = outline
			.documents
			.iter()
			.find(|spec| spec.name == document.as_str())
			.map(|spec| spec.sections.as_slice())
			.unwrap_or_default();
		for section in headings {
			let evals = anchored(&format!(
				"{document}#{}",
				Section::slug(&section.heading)
			));
			if evals.is_empty() {
				continue;
			}
			out.push_str(&format!("### {}\n\n", section.heading));
			for eval in evals {
				out.push_str(&Worksheet::entry(eval));
			}
			out.push('\n');
		}
	}
	Response::ok_text(out).xok()
}

/// How the worksheet prints one eval.
struct Worksheet;

impl Worksheet {
	fn entry(eval: &Eval) -> String {
		let mut out = format!(
			"- **{}** ({}): {}\n",
			eval.id,
			eval.levels.word(),
			eval.statement
		);
		if let Levels::Custom { l0, l1, l2, l3 } = &eval.levels {
			for (level, line) in [l0, l1, l2, l3].into_iter().enumerate() {
				out.push_str(&format!("  - {level}: {line}\n"));
			}
		}
		if let Some(note) = &eval.note {
			out.push_str(&format!("  - Note: {note}\n"));
		}
		out
	}
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
				 \x20 - Note: A judged binary eval: met or not, with no check to decide it.\n\n",
			);
	}
}
