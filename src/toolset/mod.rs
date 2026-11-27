//! The templates mounting the verbs as an agent's tools, so a role in the
//! loop works the workspace through the same routes a person runs.
//!
//! `<EvalToolset/>` in the `<Router>` beneath an agent actor gives it the
//! eight verbs a role needs, each a beet `<RouteTool>` at the verb's own path,
//! ie `eval/grade`, its definition read off the verb's route: the verb's doc
//! as its description and its declared params as its arguments, so one
//! declaration serves `--help` and the tool's schema. A call forwards to the
//! verb in the url space hosting the agent with its arguments as the verb's
//! params, validated as flags are, so the verb's workspace is the one its own route resolves, never a store a
//! toolset beside it scopes.
mod eval_toolset;
pub use eval_toolset::*;
