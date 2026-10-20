use crate::prelude::*;
use beet::prelude::*;

/// The check route that decides a binary eval, with the params it is called
/// with. A check kind is a route under `check/` in the engine's router, and a
/// new kind is a new route, nothing else: the route's signature is its only
/// declaration, and `eval/check` validates `params` against the pattern the
/// router collects for it ([`refusal`](Self::refusal)).
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

	/// The route's path in the engine's router, `check/<route>`.
	fn path(&self) -> Vec<SmolStr> {
		std::iter::once("check")
			.chain(self.route.as_str().split('/'))
			.map(SmolStr::new)
			.collect()
	}

	/// Calls the check through the router above `caller`, as any request is
	/// called, so the router refuses it as it would a person's, and reads
	/// its verdict.
	pub async fn call(&self, caller: &AsyncEntity) -> Result<CheckVerdict> {
		let router = caller
			.with_state::<AncestorQuery<Entity, With<Router>>, _>(
				|entity, routers| routers.get(entity).ok(),
			)
			.await?
			.ok_or_else(|| bevyhow!("no router above the caller"))?;
		let response =
			caller.world().entity(router).exchange(self.request()).await;
		match response.status().is_ok() {
			true => response.json::<CheckVerdict>().await,
			false => {
				bevybail!("{}", response.text().await?.trim_end())
			}
		}
	}

	/// Why the route deciding the check would refuse its params as a
	/// complete request, without calling it: a key the route does not take,
	/// a value its params type cannot read or a required param left out, each
	/// as the router's `400` names it, or no route of that name at all.
	/// `None` when it takes them. The route's pattern is the one the route
	/// tree above `caller` collects, so `--help` and this refusal read the
	/// same declaration.
	pub async fn refusal(
		&self,
		caller: &AsyncEntity,
	) -> Result<Option<String>> {
		let path = self.path();
		let request = self.request();
		let route = self.route.clone();
		caller
			.with_state::<AncestorQuery<&RouteTree>, _>(move |entity, trees| {
				match trees.get(entity).ok().and_then(|tree| tree.find(&path)) {
					Some(node) => node
						.params
						.refusal(request.params(), ParamsValidation::Complete),
					None => Some(format!(
						"no route `check/{route}` decides this check"
					)),
				}
			})
			.await
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
