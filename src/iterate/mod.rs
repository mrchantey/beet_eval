//! The loop as routes: the loop's run is a thread whose Loop actor asks
//! `eval/next` for one unit of work a turn and requests it of the url space
//! around it, and each role that works alone is a route building a thread
//! for the unit it is asked, whose agent carries the toolset of its role.
//!
//! ```bsx
//! <Router {JsonTableStore{path:"suites/acme/assets/threads"}}>
//!     <Route path="iterate" {ThreadRoute{source:Document("templates/LoopThread.bsx")}}/>
//!     <Route path="write/:document" {ThreadRoute{source:Document("templates/ClerkThread.bsx"), message:"write {document}"}}/>
//!     <Route path="grade/:document" {ThreadRoute{source:Document("templates/GraderThread.bsx"), message:"grade {document}"}}/>
//!     <Route path="build/*rubric" {ThreadRoute{source:Document("templates/BuilderThread.bsx"), message:"build {rubric}"}}/>
//! </Router>
//! ```
//!
//! A role's thread is a template, built afresh for each request, each request
//! entering a stored thread:
//!
//! ```bsx
//! <Thread {(Sequence, MountThreadStore{path:"grader", new:true})}>
//!     <CreateActor id=1 name="System" kind="System"><RoleSeed role="grader"/></CreateActor>
//!     <CreateActor id=2 name="Loop" kind="User"/>
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
//!   mounts one fixed path, `loop`, so every run continues the one loop thread,
//!   the workspace's journal of decisions, each run a session of its Loop
//!   actor.
//! - **One unit per turn**: each time round [`NextUnit`] appends its
//!   decision to the loop thread, `<verb> <target>: <why>`, and requests one
//!   unit, and the next time round `eval/next` is asked again of what that
//!   unit left.
//! - **Every unit names its decision**: the unit is requested with
//!   `spawned_by` naming the loop thread and the decision post, so a unit
//!   thread the request creates records it as its creator, and every join
//!   of a unit thread names the decision that asked for it. A unit run by
//!   hand names none.
//! - **Shape before substance**: the order is [`NextStep`](crate::prelude::NextStep)'s, so a document
//!   missing or failing its shape checks is written before anything is
//!   graded or built.
//! - **A sitting stops the loop**: a coach's sitting is a conversation with
//!   a person, held by `RunThread` in an entry with a surface
//!   ([`HoldClaim`](crate::prelude::HoldClaim)), never a unit of work, so when `next` names one the
//!   loop records the decision and stops, saying which claim to sit down
//!   with.
//! - **Continuation by target**: a role's seed is its procedure,
//!   [`RoleSeed`], the initial posts of a fresh thread, and each request
//!   posts the unit it asks, its route's `message` filled from the path
//!   param, once its actors have joined. The clerk continues its thread
//!   about one document and the builder about one rubric, each by the target
//!   its mount path names (`clerk/{document}`), keeping the seed it started
//!   with, so an edited seed reaches a new thread (`--new`). The grader's
//!   mount is `new:true`, so it judges fresh every time, unanchored by its
//!   earlier grades.
//! - **A grader cannot write**: its store tools are
//!   `<StoreToolset read_only=true/>`, and its calls are answered by its own
//!   router alone, which has no `write-text`; its grades go through
//!   `eval/grade`, which writes through the workspace store the surrounding
//!   url space resolves.
//! - **A builder writes only the build**: its store tools sit under a
//!   `DirPath("dist")`, the build directory.
//! - **The coach holds claims, never asks**: a decision ask whose section no
//!   claim names is the clerk's to raise as a claim, and the coach's sitting
//!   on a claim ends once its assignment is written to the claim's `test`,
//!   [`AssignmentWritten`](crate::prelude::AssignmentWritten).
//! - **Every role runs by hand**: a role is a route, so
//!   `beet-eval --entry=iterate.bsx grade product` runs one unit.
mod next_unit;
mod role_seed;
pub use next_unit::*;
pub use role_seed::*;
