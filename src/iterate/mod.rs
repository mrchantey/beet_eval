//! The loop as routes: the loop's run is a thread whose Loop actor asks
//! `eval/next` for one unit of work a turn and requests it of the host url
//! space, and each role is a route building a thread for the unit it is
//! asked, whose agent carries the toolset of its role.
//!
//! ```bsx
//! <Route path="iterate" {ThreadRoute{template:"LoopThread", metadata:{journal:"loop"}}}/>
//! <Route path="write/:document" {ThreadRoute{template:"ClerkThread", metadata:{document:"{document}"}}}/>
//! <Route path="grade/:document" {ThreadRoute{template:"GraderThread"}}/>
//! <Route path="build/*rubric" {ThreadRoute{template:"BuilderThread", metadata:{rubric:"{rubric}"}}}/>
//! <Route path="coach/:claim" {ThreadRoute{template:"CoachThread", metadata:{claim:"{claim}"}}}/>
//! ```
//!
//! A role's thread is a template, built afresh for each request, each request
//! one session of a stored thread:
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
//! - **The run is a thread**: the `iterate` route's `LoopThread` holds one
//!   Loop actor whose behavior is a `RepeatTimes` over [`NextUnit`], and
//!   declares fixed metadata, so every run continues the one loop thread,
//!   the workspace's journal of decisions, each run a session of it.
//! - **One unit per turn**: each time round [`NextUnit`] appends its
//!   decision to the loop thread, `<verb> <target>: <why>`, and requests one
//!   unit, and the next time round `eval/next` is asked again of what that
//!   unit left.
//! - **Every unit names its decision**: the unit is requested with
//!   `spawned_by` naming the loop thread and the decision post, so a unit
//!   thread the request creates records it as its creator, and every session
//!   of a unit thread names the decision that asked for it in its opened
//!   post. A unit run by hand names none.
//! - **Shape before substance**: the order is [`NextStep`]'s, so a document
//!   missing or failing its shape checks is written before anything is
//!   graded or built.
//! - **Continuation by target**: a role's seed is its procedure,
//!   [`RoleSeed`], the initial posts of a fresh thread, and each request
//!   posts the unit it asks, a `<RequestPost>` reading the path param, as its
//!   session opens. The clerk
//!   continues its thread about one document, the builder about one rubric
//!   and the coach about one claim, each by the metadata its route declares,
//!   keeping the seed it started with, so an edited seed reaches a new thread
//!   (`--new`). The grader declares none, so it judges fresh every time,
//!   unanchored by its earlier grades.
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
