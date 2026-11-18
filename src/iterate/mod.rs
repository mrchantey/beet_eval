//! The loop as routes: an `iterate` route asks `eval/next` for one unit of
//! work a turn and requests it of the host url space, and each role is a
//! route building a thread for the unit it is asked, whose agent carries the
//! toolset of its role.
//!
//! ```bsx
//! <Route path="iterate" {ExchangeSequence}>
//!     <Repeat {OutcomeOverload}><NextUnit/></Repeat>
//! </Route>
//! <Route path="write/:document" {ThreadRoute{template:"ClerkThread"}}/>
//! <Route path="grade/:document" {ThreadRoute{template:"GraderThread"}}/>
//! <Route path="build/*rubric" {ThreadRoute{template:"BuilderThread"}}/>
//! <Route path="coach/:claim" {ThreadRoute{template:"CoachThread"}}/>
//! ```
//!
//! A role's thread is a template, built afresh for each request:
//!
//! ```bsx
//! <Thread {(Sequence, MountThreadStore{path:"packages/acme/assets/threads/grader"})}>
//!     <CreateActor id=1 name="System" kind="System"><RoleSeed role="grader"/></CreateActor>
//!     <CreateActor id=2 name="Loop" kind="User"><RequestPost text="grade {document}"/></CreateActor>
//!     <RepeatWhileFunctionCallOutput>
//!         <CreateActor id=3 name="Grader" kind="Agent" {ModelStreamer{provider:OpenAi}}>
//!             <Router><StoreToolset read_only=true/><EvalToolset/></Router>
//!         </CreateActor>
//!     </RepeatWhileFunctionCallOutput>
//! </Thread>
//! ```
//!
//! The rules of the loop are its structure:
//!
//! - **One unit per turn**: each time round [`NextUnit`] requests one unit,
//!   and the next time round `eval/next` is asked again of what that unit
//!   left.
//! - **Shape before substance**: the order is [`NextStep`]'s, so a document
//!   missing or failing its shape checks is written before anything is
//!   graded or built.
//! - **A thread per unit**: a role's seed is its procedure, [`RoleSeed`], and
//!   the unit its route was asked, a `<RequestPost>` reading the path param,
//!   both read once as the thread builds, so each unit is a thread of its
//!   own, persisted under the workspace package and resumed by its seed when
//!   the same unit comes round again.
//! - **A grader cannot write**: its store tools are
//!   `<StoreToolset read_only=true/>`, and its calls are answered by its own
//!   router alone, which has no `write-text`; its grades go through
//!   `eval/grade`, which writes through the workspace store the host's route
//!   resolves.
//! - **A builder writes only the build**: its store tools sit under a
//!   `DirPath("dist")`, the build directory.
//! - **The coach holds claims, never asks**: a decision ask whose section no
//!   claim names is the clerk's to raise as a claim, and the coach's sitting
//!   on a claim ends once its assignment is written to the claim's `test`,
//!   [`AssignmentWritten`].
//! - **Every role runs by hand**: a role is a route, so
//!   `beet-eval --entry=iterate.bsx grade product` runs one unit.
mod next_unit;
mod role_seed;
pub use next_unit::*;
pub use role_seed::*;
