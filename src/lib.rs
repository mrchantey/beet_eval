//! Evaluates a subject against packages of evals and rubrics: checks, grades,
//! results and the next unit of work, with markdown documents as the first
//! subject and Office forms as the renderer. `README.md` fixes the words; the
//! modules carry the law, in the order a run reads it:
//!
//! - [`eval`]: the core, subject agnostic. An [`Eval`](prelude::Eval) is one
//!   statement judged on the four [`Levels`](prelude::Levels) or decided by a
//!   [`CheckRef`](prelude::CheckRef), a [`Rubric`](prelude::Rubric) cites
//!   evals at levels for one reader, [`Grade`](prelude::Grade) rows carry a
//!   grader's levels with verbatim evidence, one
//!   [`Results`](prelude::Results) is read by every rubric, and
//!   [`NextStep`](prelude::NextStep) names the next unit of work. Packages and
//!   workspaces are stores with a
//!   [`PackageManifest`](prelude::PackageManifest) or a
//!   [`Workspace`](prelude::Workspace) manifest.
//! - [`markdown`]: the first subject, documents as markdown files with
//!   frontmatter, sections, csv data blocks and asks, laid out by a document
//!   package's [`Outline`](prelude::Outline), and the params of the check
//!   kinds that decide their shape.
//! - [`form`]: the renderer, a reader package's
//!   [`RenderSpec`](prelude::RenderSpec) and the
//!   [`FillSpec`](prelude::FillSpec) a builder writes against it.
//! - [`coach`]: the workspace package's claims register and the document
//!   package's coach actions.
// the harness main for `cargo test --lib`; cfg gated so a plain build does not
// need the facade's `testing` feature
#[cfg(test)]
beet::test_main!();

pub mod coach;
pub mod eval;
pub mod form;
pub mod markdown;
mod plugin;
mod text_type;

/// Exports the most commonly used items.
pub mod prelude {
	pub use crate::coach::*;
	pub use crate::eval::*;
	pub use crate::form::*;
	pub use crate::markdown::*;
	pub use crate::plugin::*;
}


#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::exports::bevy::reflect::Typed;
	use beet::prelude::*;

	/// Every `json <Type>` example in the README deserializes into its type,
	/// validates against the schema the type derives and serializes back
	/// unchanged, so the README shows exactly the stored form, and every stored
	/// shape has an example.
	#[beet::test]
	async fn readme_examples_are_the_stored_form() {
		let mut shown = Vec::new();
		for (name, json) in readme_examples() {
			match name {
				"PackageManifest" => {
					stored_form::<PackageManifest>(name, &json).await
				}
				"Outline" => stored_form::<Outline>(name, &json).await,
				"Eval" => stored_form::<Eval>(name, &json).await,
				"Rubric" => stored_form::<Rubric>(name, &json).await,
				"RenderSpec" => stored_form::<RenderSpec>(name, &json).await,
				"CoachAction" => stored_form::<CoachAction>(name, &json).await,
				"Claim" => stored_form::<Claim>(name, &json).await,
				"Workspace" => stored_form::<Workspace>(name, &json).await,
				"Grade" => stored_form::<Grade>(name, &json).await,
				"Results" => stored_form::<Results>(name, &json).await,
				"NextStep" => stored_form::<NextStep>(name, &json).await,
				"FillSpec" => stored_form::<FillSpec>(name, &json).await,
				other => panic!(
					"the README shows `{other}`, which is no stored shape"
				),
			}
			shown.push(name);
		}
		shown.sort();
		shown.dedup();
		shown.xpect_eq(vec![
			"Claim",
			"CoachAction",
			"Eval",
			"FillSpec",
			"Grade",
			"NextStep",
			"Outline",
			"PackageManifest",
			"RenderSpec",
			"Results",
			"Rubric",
			"Workspace",
		]);
	}

	/// The README's fences whose info string is `json <Type>`, with their
	/// bodies.
	fn readme_examples() -> Vec<(&'static str, String)> {
		let mut examples = Vec::new();
		let mut lines = include_str!("../README.md").lines();
		while let Some(line) = lines.next() {
			if let Some(name) = line.strip_prefix("```json ") {
				let body = lines
					.by_ref()
					.take_while(|line| *line != "```")
					.collect::<Vec<_>>()
					.join("\n");
				examples.push((name, body));
			}
		}
		examples
	}

	async fn stored_form<T: Typed + Serialize + DeserializeOwned>(
		name: &str,
		json: &str,
	) {
		let written =
			MediaBytes::new_json(json).deserialize::<Value>().unwrap();
		ValueSchema::of::<T>()
			.assert_valid(name, &mut written.clone())
			.await
			.unwrap();
		written
			.clone()
			.into_serde::<T>()
			.unwrap()
			.xmap(Value::from_serde)
			.unwrap()
			.xpect_eq(written);
	}
}
