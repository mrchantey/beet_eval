use beet::prelude::*;

/// A request one route makes of another, its params a serializable value
/// spelled as the flags a command line would write: how `eval/results` calls a
/// check kind in its own url space ([`Self::exchange`]), and how a tool calls
/// its verb in the url space hosting its agent ([`Self::exchange_host`]).
pub(crate) struct RouteCall {
	request: Request,
}

impl RouteCall {
	/// A `GET` of `path`.
	pub fn new(path: impl AsRef<str>) -> Self {
		Self {
			request: Request::get(path.as_ref()),
		}
	}

	/// Adds every field of `params`, a map, as a flag in kebab case: a list
	/// repeats its flag, a null or an empty list is left out.
	pub fn with_params(mut self, params: &Value) -> Self {
		if let Value::Map(map) = params {
			for (key, value) in map.0.iter() {
				let flag = key.replace('_', "-");
				match value {
					Value::Null => {}
					Value::List(items) => {
						for item in items {
							self.request.insert_param(
								flag.clone(),
								Self::flag_value(item),
							);
						}
					}
					value => {
						self.request.insert_param(flag, Self::flag_value(value))
					}
				}
			}
		}
		self
	}

	/// Adds the fields of a serializable params type, as [`Self::with_params`].
	pub fn with_params_of(self, params: &impl Serialize) -> Result<Self> {
		Value::from_serde(params)?
			.xmap(|params| self.with_params(&params))
			.xok()
	}

	/// Asks for an answer in `media_type`.
	pub fn with_accept(mut self, media_type: MediaType) -> Self {
		self.request = self.request.with_header::<header::Accept>(media_type);
		self
	}

	/// The request as built.
	pub fn into_request(self) -> Request { self.request }

	/// The answer of the url space `caller` is a route of, refusing the
	/// request exactly as it would a person's: how a verb calls a verb.
	pub async fn exchange(self, caller: &AsyncEntity) -> Result<Response> {
		let space = caller
			.with_state::<(Query<&ChildOf>, Query<&PathPartial>), _>(
				|entity, (parents, partials)| {
					PathPattern::namespace_root(entity, &parents, &partials)
				},
			)
			.await?;
		self.exchange_with(caller, space).await
	}

	/// The answer of the url space hosting the agent whose tool `tool` is
	/// ([`ThreadQuery::agent_host`]), never the agent's own: its tools are
	/// the routes of the `<Router>` beneath it, and the verbs a tool forwards
	/// to are routes beside the agent.
	pub async fn exchange_host(self, tool: &AsyncEntity) -> Result<Response> {
		let host = tool
			.with_state::<ThreadQuery, _>(|entity, query| {
				query.agent_host(entity)
			})
			.await??;
		self.exchange_with(tool, host).await
	}

	/// The answer of the url space rooted at `space`.
	async fn exchange_with(
		self,
		caller: &AsyncEntity,
		space: Entity,
	) -> Result<Response> {
		caller
			.world()
			.entity(space)
			.exchange(self.request)
			.await
			.xok()
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
