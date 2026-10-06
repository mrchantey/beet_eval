//! The plugin registering every reflect type this crate defines, so an entry,
//! a schema registry or a form resolves them by name.
use crate::prelude::*;
use beet::prelude::*;

/// This crate's types: the rows of every table, the typed documents of every
/// package and workspace, and the params of every check kind.
#[derive(Default)]
pub struct EvalPlugin;

impl Plugin for EvalPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<Eval>()
			.register_type::<Rubric>()
			.register_type::<Grade>()
			.register_type::<Results>()
			.register_type::<NextStep>()
			.register_type::<PackageManifest>()
			.register_type::<Workspace>()
			.register_type::<DocumentTemplate>()
			.register_type::<MarkdownDocument>()
			.register_type::<RenderSpec>()
			.register_type::<FillSpec>()
			.register_type::<Claim>()
			.register_type::<CoachAction>()
			.register_type::<DocumentCheckParams>()
			.register_type::<FrontmatterCheckParams>()
			.register_type::<H1CheckParams>()
			.register_type::<TaglineCheckParams>()
			.register_type::<SummaryCheckParams>()
			.register_type::<AgreesCheckParams>()
			.register_type::<AsksCheckParams>()
			.register_type::<SectionsCheckParams>()
			.register_type::<BlockCheckParams>();
	}
}
