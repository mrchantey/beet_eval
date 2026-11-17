use crate::prelude::*;
use beet::prelude::*;

/// `<EvalToolset/>`: the verbs a role works the loop with, as tools: `check`,
/// `results`, `next`, `worksheet`, `project`, `blocks`, `grade` and `build`,
/// each forwarding to `eval/<verb>` and answering its markdown report, a
/// refusal answering as the error it is.
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
/// the agent ([`ThreadQuery::agent_host`]), so a tool's `eval/grade` is the
/// host's, never a route the agent could call itself.
#[template]
pub fn EvalToolset() -> impl Bundle {
	children![
		route::exchange("check", CheckTool),
		route::exchange("results", ResultsTool),
		route::exchange("next", NextTool),
		route::exchange("worksheet", WorksheetTool),
		route::exchange("project", ProjectTool),
		route::exchange("blocks", BlocksTool),
		route::exchange("grade", GradeTool),
		route::exchange("build", BuildTool),
	]
}

/// The input of a tool whose verb takes nothing: it reads the whole
/// workspace.
#[derive(Debug, Default, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct NoArguments {}

/// Checks the workspace against the law of evals and rubrics: every eval's
/// levels, anchor and check params, every rubric's citations, each problem
/// named with where it is. Call it after changing a row.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
pub async fn CheckTool(cx: ActionContext<NoArguments>) -> Result<Response> {
	forward(&cx.caller, "eval/check", &cx.input).await
}

/// The results of the last run: every check passing or failing, the grades
/// merged, and each rubric's citations met, failing or awaiting a grade.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
pub async fn ResultsTool(cx: ActionContext<NoArguments>) -> Result<Response> {
	forward(&cx.caller, "eval/results", &cx.input).await
}

/// The next unit of work and why, with the documents and rubrics it was
/// chosen from.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
pub async fn NextTool(cx: ActionContext<NoArguments>) -> Result<Response> {
	forward(&cx.caller, "eval/next", &cx.input).await
}

/// The worksheet: every judged eval anchored in a document, by section, with
/// its statement and the lines of each level, what a writer writes towards
/// and a grader grades against.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
pub async fn WorksheetTool(
	cx: ActionContext<WorksheetParams>,
) -> Result<Response> {
	forward(&cx.caller, "eval/worksheet", &cx.input).await
}

/// The projection of a reader rubric, `<package>/<rubric>`: for every heading
/// of its form, the evals it cites, what is written at each anchor today and
/// the heading's structural lines, the brief a fill spec is written from.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
pub async fn ProjectTool(cx: ActionContext<ProjectParams>) -> Result<Response> {
	forward(&cx.caller, "eval/project", &cx.input).await
}

/// The named data blocks of the documents, or one block's body by its name.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
pub async fn BlocksTool(cx: ActionContext<BlocksParams>) -> Result<Response> {
	forward(&cx.caller, "eval/blocks", &cx.input).await
}

/// Grades one judged eval: its level by the eval's own lines and a verbatim
/// quote of at most twenty-five words from the anchored section as evidence.
/// Refused, with the reason, for an unknown or checked eval, a level the eval
/// does not admit, or evidence that is not written there word for word.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
pub async fn GradeTool(cx: ActionContext<Grading>) -> Result<Response> {
	forward(&cx.caller, "eval/grade", &cx.input).await
}

/// Builds a reader's form, `<package>/<rubric>`, from the fill spec at
/// `<package>/<rubric>.fill.json` in the build directory, writing the filled
/// form and its cells beside it.
#[action]
#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
pub async fn BuildTool(cx: ActionContext<BuildParams>) -> Result<Response> {
	// the rubric is the greedy path after `build`, never a flag
	RouteCall::new(format!("eval/build/{}", cx.input.rubric))
		.with_accept(MediaType::Markdown)
		.exchange_host(&cx.caller)
		.await
}

/// The verb at `path` in the url space hosting the agent, called with
/// `params` as its flags, answering markdown, an agent's reading form.
async fn forward(
	caller: &AsyncEntity,
	path: &str,
	params: &impl Serialize,
) -> Result<Response> {
	RouteCall::new(path)
		.with_params_of(params)?
		.with_accept(MediaType::Markdown)
		.exchange_host(caller)
		.await
}
