//! The templates mounting the verbs as an agent's tools, so a role in the
//! loop works the workspace through the same routes a person runs.
//!
//! A tool is a route with a typed input and a description, which the agent
//! sees as a tool definition: `<EvalToolset/>` in the `<Router>` beneath an
//! agent actor gives it the eight verbs a role needs, each forwarding to
//! `eval/<verb>` in the url space hosting the agent, so the verb's workspace
//! is the one its own route resolves, never a store a toolset beside it
//! scopes. The input of a tool taking params is the verb's own
//! params type, ie [`Grading`], so one declaration serves `--help` and the
//! tool's schema.
mod eval_toolset;
pub use eval_toolset::*;
