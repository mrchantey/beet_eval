use crate::prelude::*;
use beet::exports::bevy::reflect::ReflectRef;
use beet::exports::bevy::reflect::structs::Struct;
use beet::prelude::*;

/// The params of [`FrontmatterCheck`].
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct FrontmatterCheckParams {
	/// The keys each must carry, ie [`DocumentSet::META_KEYS`].
	pub keys: Vec<SmolStr>,
	/// The documents' names.
	pub documents: Vec<SmolStr>,
}

impl FrontmatterCheckParams {
	/// The verdict on `documents`.
	pub fn decide(&self, documents: &mut DocumentSet) -> CheckVerdict {
		let mut problems = Vec::new();
		let mut failing = Vec::new();
		for name in &self.documents {
			let path = documents.path_name(name);
			let document = documents.get(name).cloned();
			let meta = document.as_ref().and_then(|document| {
				documents.query(|query| query.meta(document.root).cloned())
			});
			let problem = match document {
				None => Some(format!("{path}: missing")),
				Some(document) => match &meta {
					None => Some(match document.problems.first() {
						Some(problem) => format!("{path}: {problem}"),
						None => format!("{path}: no frontmatter"),
					}),
					Some(meta) => {
						let missing = self
							.keys
							.iter()
							.filter(|key| !Self::has_value(meta, key))
							.map(SmolStr::as_str)
							.collect::<Vec<_>>();
						(!missing.is_empty())
							.then(|| format!("{path}: {}", missing.join(", ")))
					}
				},
			};
			if let Some(problem) = problem {
				problems.push(problem);
				failing.push(name.clone());
			}
		}
		match problems.is_empty() {
			true => CheckVerdict::pass(format!(
				"{} on {} documents",
				self.keys.join(", "),
				self.documents.len()
			)),
			false => CheckVerdict::fail(problems.join("; "), failing),
		}
	}

	/// Whether the frontmatter gives `key` a value: an option set, a list
	/// with an item, text that is not empty.
	fn has_value(meta: &PageMeta, key: &str) -> bool {
		let Some(field) = Struct::field(meta, key) else {
			return false;
		};
		match field.reflect_ref() {
			ReflectRef::Enum(value) => value.variant_name() != "None",
			ReflectRef::List(list) => list.len() > 0,
			_ => field
				.try_downcast_ref::<String>()
				.is_none_or(|text| !text.is_empty()),
		}
	}
}

/// `check/frontmatter`: passes when every document resolves and its
/// frontmatter carries every key with a value.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
#[require(
	PathPartial = PathPartial::new("frontmatter"),
	ParamsPartial = ParamsPartial::new::<FrontmatterCheckParams>()
)]
pub async fn FrontmatterCheck(cx: ActionContext<Request>) -> Result<Response> {
	DocumentSet::answer_check(&cx, FrontmatterCheckParams::decide).await
}
