use crate::prelude::*;
use beet::prelude::*;

/// The check route that decides a binary eval, with the params it is called
/// with. A check kind is a route under `check/` in the engine's router, and a
/// new kind is a new route, nothing else: `eval/check` validates `params`
/// against that route's [`ParamsPartial`].
///
/// ```json
/// { "route": "sections", "params": { "document": "brand", "headings": ["Name and tagline", "Positioning"] } }
/// ```
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct CheckRef {
	/// The route below `check/`, ie `sections` for `check/sections`.
	pub route: RelPath,
	/// The route's params as its flags name them, each a string or a list of
	/// strings, ie `{"document": "brand"}` for `--document=brand`.
	pub params: Value,
}

impl CheckRef {
	/// A reference to the check route `route` called with `params`, any
	/// serializable params type, ie [`SectionsCheckParams`].
	pub fn new(route: &str, params: impl Serialize) -> Result<Self> {
		Self {
			route: RelPath::new(route),
			params: Value::from_serde(params)?,
		}
		.xok()
	}
}

/// What one check route decided about one eval in one run: a row of
/// [`Results::checks`].
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct CheckOutcome {
	/// The checked eval.
	pub eval: EvalId,
	/// Whether the check passed, awarding [`EvalLevel::SOUND`], or failed,
	/// awarding [`EvalLevel::MISSING`].
	pub pass: bool,
	/// What the route found, ie `6 sections in order` or `missing: Voice`.
	pub detail: String,
}
