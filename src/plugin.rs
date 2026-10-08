//! The plugin registering every reflect type this crate defines, so an entry,
//! a schema registry or a form resolves them by name.
use crate::prelude::*;
use beet::prelude::*;

/// This crate's types: the rows of every table, the typed documents of every
/// package and workspace, the check kinds with their params, the verbs, and
/// the `<CheckRoutes/>` and `<EvalRoutes/>` mounting them.
#[derive(Default)]
pub struct EvalPlugin;

impl Plugin for EvalPlugin {
	fn build(&self, app: &mut App) {
		// the text types read validated wherever they are authored, a flag
		// or a route segment included
		LiteralParser::register::<EvalId>(EvalId::literal_parser(
			"an eval id, ie market.competitors-named",
		));
		LiteralParser::register::<Address>(Address::literal_parser(
			"an address, ie product#pricing",
		));
		LiteralParser::register::<RubricRef>(RubricRef::literal_parser(
			"a rubric, ie course/01-business-plan",
		));
		LiteralParser::register::<SourceTag>(SourceTag::literal_parser(
			"a source tag, ie ref:sba",
		));
		LiteralParser::register::<CellRef>(CellRef::literal_parser(
			"a cell, ie t1r2c3 or Sheet!A1",
		));
		app.register_type::<Eval>()
			.register_type::<Rubric>()
			.register_type::<Grade>()
			.register_type::<Results>()
			.register_type::<NextStep>()
			.register_type::<PackageManifest>()
			.register_type::<Workspace>()
			.register_type::<Outline>()
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
			.register_type::<BlockCheckParams>()
			.register_template::<CheckRoutes>()
			.register_type::<DocumentCheck>()
			.register_type::<FrontmatterCheck>()
			.register_type::<H1Check>()
			.register_type::<TaglineCheck>()
			.register_type::<SummaryCheck>()
			.register_type::<AgreesCheck>()
			.register_type::<AsksCheck>()
			.register_type::<SectionsCheck>()
			.register_type::<BlockCheck>()
			.register_template::<EvalRoutes>()
			.register_type::<EvalCheck>()
			.register_type::<EvalResults>()
			.register_type::<EvalNext>()
			.register_type::<EvalBlocks>()
			.register_type::<EvalWorksheet>()
			.register_type::<EvalProject>()
			.register_type::<EvalNew>()
			.register_type::<EvalGrade>()
			.register_type::<EvalBuild>()
			.register_type::<EvalCells>()
			.register_type::<EvalPut>()
			.register_type::<EvalDrop>();
	}
}
