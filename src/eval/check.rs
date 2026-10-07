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

	/// The request calling the route: each param a flag in kebab case, a list
	/// repeating its flag.
	pub fn request(&self) -> Request {
		let mut request = Request::get(format!("/check/{}", self.route));
		if let Value::Map(map) = &self.params {
			for (key, value) in map.0.iter() {
				let flag = key.replace('_', "-");
				match value {
					Value::List(items) => {
						for item in items {
							request.insert_param(
								flag.clone(),
								Self::flag_value(item),
							);
						}
					}
					value => {
						request.insert_param(flag, Self::flag_value(value))
					}
				}
			}
		}
		request
	}

	/// The route deciding the check, found in the route tree above `caller`.
	pub async fn route_entity(&self, caller: &AsyncEntity) -> Result<Entity> {
		let path = std::iter::once("check")
			.chain(self.route.as_str().split('/'))
			.map(SmolStr::new)
			.collect::<Vec<_>>();
		caller
			.with_state::<AncestorQuery<&RouteTree>, _>(move |entity, trees| {
				trees
					.get(entity)
					.ok()
					.and_then(|tree| tree.find(&path))
					.map(|node| node.entity)
			})
			.await?
			.ok_or_else(|| {
				bevyhow!("no route `check/{}` decides this check", self.route)
			})
	}

	/// Calls the check's route and reads its verdict.
	pub async fn call(&self, caller: &AsyncEntity) -> Result<CheckVerdict> {
		let route = self.route_entity(caller).await?;
		caller
			.world()
			.entity(route)
			.call::<Request, Response>(self.request())
			.await?
			.json::<CheckVerdict>()
			.await
	}

	/// Checks the params against the params the route declares, without
	/// calling it.
	pub async fn validate(&self, caller: &AsyncEntity) -> Result {
		let route = self.route_entity(caller).await?;
		let request = self.request();
		caller
			.world()
			.entity(route)
			.get::<ParamsPartial, _>(move |partial| {
				partial.validate(request.params())
			})
			.await
			.map_err(|_| {
				bevyhow!("the route `check/{}` declares no params", self.route)
			})?
	}

	/// A scalar as a flag's value.
	fn flag_value(value: &Value) -> String {
		match value {
			Value::Str(text) => text.to_string(),
			Value::Null => String::new(),
			other => other.to_string(),
		}
	}
}

/// What a check route answers, as JSON: whether its eval passes, what it
/// found, and the documents whose shape a failure is laid at, which the
/// triage reads. An open ask is no shape failure, counted on its own.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct CheckVerdict {
	/// Whether the check passed.
	pub pass: bool,
	/// What the route found.
	pub detail: String,
	/// The documents a failure concerns, by name; empty on a pass.
	pub documents: Vec<SmolStr>,
}

impl CheckVerdict {
	/// A pass, with what was found.
	pub fn pass(detail: impl Into<String>) -> Self {
		Self {
			pass: true,
			detail: detail.into(),
			documents: Vec::new(),
		}
	}

	/// A failure laid at `documents`.
	pub fn fail(
		detail: impl Into<String>,
		documents: impl IntoIterator<Item = impl Into<SmolStr>>,
	) -> Self {
		Self {
			pass: false,
			detail: detail.into(),
			documents: documents.into_iter().map(Into::into).collect(),
		}
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
