use beet::prelude::*;

/// `<EvalToolset/>`: the verbs a role works the loop with, as tools:
/// `eval/check`, `eval/results`, `eval/next`, `eval/worksheet`,
/// `eval/project`, `eval/blocks`, `eval/grade`, `eval/cells`, `eval/answers`
/// and `eval/build`, each a [`RouteTool`] answering the verb's markdown
/// report, a refusal answering as the error it is. `eval/approve` is none of them: a
/// person approves a form's answers, never a role.
///
/// ```bsx
/// <CreateActor name="Grader" kind="Agent" {ModelStreamer{provider:OpenAi}}>
///     <Router>
///         <EvalToolset/>
///         <StoreToolset read_only=true/>
///     </Router>
/// </CreateActor>
/// ```
///
/// The tools are routes of the agent's own url space, the `<Router>` beneath
/// it, while the verbs they forward to are routes of the url space hosting
/// the agent, so a tool's `eval/grade` is the host's, named as a person runs
/// it, never a route the agent could call itself.
#[template]
pub fn EvalToolset() -> impl Bundle {
	rsx! {
		<RouteTool path="eval/check"/>
		<RouteTool path="eval/results"/>
		<RouteTool path="eval/next"/>
		<RouteTool path="eval/worksheet"/>
		<RouteTool path="eval/project"/>
		<RouteTool path="eval/blocks"/>
		<RouteTool path="eval/grade"/>
		<RouteTool path="eval/cells"/>
		<RouteTool path="eval/answers"/>
		<RouteTool path="eval/build"/>
	}
}
