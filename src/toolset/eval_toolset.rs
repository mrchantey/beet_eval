use beet::prelude::*;

/// `<EvalToolset/>`: the verbs a role works the loop with, as tools:
/// `eval/check`, `eval/results`, `eval/next`, `eval/worksheet`,
/// `eval/project`, `eval/blocks`, `eval/grade` and `eval/build`, each a
/// [`HostTool`] answering the verb's markdown report, a refusal answering as
/// the error it is.
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
		<HostTool path="eval/check"/>
		<HostTool path="eval/results"/>
		<HostTool path="eval/next"/>
		<HostTool path="eval/worksheet"/>
		<HostTool path="eval/project"/>
		<HostTool path="eval/blocks"/>
		<HostTool path="eval/grade"/>
		<HostTool path="eval/build"/>
	}
}
