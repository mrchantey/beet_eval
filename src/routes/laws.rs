use crate::prelude::*;
use beet::prelude::*;

/// The rules every row is held to, each answering what is wrong.
pub(crate) struct Laws;

impl Laws {
	/// An eval's laws and its check's params against the route that decides
	/// it, found above `caller`.
	pub async fn eval_row(
		caller: &AsyncEntity,
		eval: &Eval,
		outline: Option<&Outline>,
	) -> Vec<String> {
		let mut problems = Self::eval(eval, outline);
		if let Some(check) = eval.levels.check() {
			if let Err(err) = check.validate(caller).await {
				problems.push(format!("check/{}: {err}", check.route));
			}
		}
		problems
	}

	/// A rubric's prose and its citations.
	pub fn rubric(workspace: &LoadedWorkspace, rubric: &Rubric) -> Vec<String> {
		let mut problems = Self::prose(rubric);
		problems.extend(
			rubric
				.citations()
				.filter_map(|citation| Self::citation(workspace, citation)),
		);
		problems
	}

	/// An eval's prose, levels and anchors.
	fn eval(eval: &Eval, outline: Option<&Outline>) -> Vec<String> {
		let mut problems = Self::prose(eval);
		if !Self::one_sentence(&eval.statement) {
			problems.push("the statement must be exactly one sentence ending in a full stop".into());
		}
		if let Levels::Custom { l0, l1, l2, l3 } = &eval.levels {
			for (level, line) in [l0, l1, l2, l3].into_iter().enumerate() {
				if !line.trim_end().ends_with('.') {
					problems.push(format!(
						"level {level} must be one sentence ending in a full stop"
					));
				}
			}
		}
		if eval.sources.is_empty() {
			problems.push("an eval needs at least one source".into());
		}
		let Some(outline) = outline else {
			return problems;
		};
		if let Some(anchor) = &eval.anchor {
			if let Some(problem) = Self::address(outline, anchor) {
				problems.push(format!("anchor {anchor}: {problem}"));
			}
		}
		// the addresses a check's params name are anchors too
		if let Some(check) = eval.levels.check() {
			let named = match check.route.as_str() {
				"block" => check
					.params
					.clone()
					.into_serde::<BlockCheckParams>()
					.ok()
					.map(|params| params.section),
				"agrees" => check
					.params
					.clone()
					.into_serde::<AgreesCheckParams>()
					.ok()
					.map(|params| params.section),
				_ => None,
			};
			if let Some(address) = named {
				if let Some(problem) = Self::address(outline, &address) {
					problems.push(format!(
						"check/{} section {address}: {problem}",
						check.route
					));
				}
			}
		}
		problems
	}

	/// What is wrong with `address` against the outline: a document or a
	/// section it does not declare.
	pub fn address(outline: &Outline, address: &Address) -> Option<String> {
		let Some(document) = outline
			.documents
			.iter()
			.find(|document| document.name == address.document_name())
		else {
			return Some(
				"names a document the outline does not declare".into(),
			);
		};
		match address.section() {
			Some(slug)
				if !document
					.sections
					.iter()
					.any(|section| Address::slug(&section.heading) == slug) =>
			{
				Some("names a section the outline does not declare".into())
			}
			_ => None,
		}
	}

	/// A citation of an eval the workspace defines, at a level it can reach.
	fn citation(
		workspace: &LoadedWorkspace,
		citation: &Citation,
	) -> Option<String> {
		let Some(packaged) = workspace.eval(&citation.eval) else {
			return Some(format!("unknown eval {}", citation.eval));
		};
		match (citation.level.get(), &packaged.eval.levels) {
			(0, _) => Some(format!("{}: level must be 1 to 3", citation.eval)),
			(level, Levels::Binary { .. }) if level != 2 => Some(format!(
				"{} is binary and can only be cited at level 2",
				citation.eval
			)),
			_ => None,
		}
	}

	/// The prose rules over every text a row holds: no em or en dash.
	pub fn prose(row: &impl Serialize) -> Vec<String> {
		let mut texts = Vec::new();
		if let Ok(value) = Value::from_serde(row) {
			Self::texts(&value, &mut texts);
		}
		texts
			.into_iter()
			.filter(|text| text.contains(['\u{2014}', '\u{2013}']))
			.map(|text| {
				format!(
					"em or en dash in '{}'",
					text.chars().take(60).collect::<String>()
				)
			})
			.collect()
	}

	fn texts(value: &Value, out: &mut Vec<String>) {
		match value {
			Value::Str(text) => out.push(text.to_string()),
			Value::List(items) => {
				for item in items {
					Self::texts(item, out);
				}
			}
			Value::Map(map) => {
				for item in map.0.values() {
					Self::texts(item, out);
				}
			}
			_ => {}
		}
	}

	/// Exactly one sentence ending in a full stop.
	fn one_sentence(text: &str) -> bool {
		text.ends_with('.') && !text.contains(". ")
	}
}
