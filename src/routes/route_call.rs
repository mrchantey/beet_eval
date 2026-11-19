use beet::prelude::*;

/// A request one route makes of another in the url space it is a route of,
/// refused exactly as a person's would be: how `eval/results` calls a check
/// kind and how the loop's `NextUnit` requests a role.
pub(crate) struct RouteCall {
	request: Request,
}

impl RouteCall {
	/// The answer of the url space `caller` is a route of.
	pub async fn exchange(self, caller: &AsyncEntity) -> Result<Response> {
		let space = caller
			.with_state::<(Query<&ChildOf>, Query<&PathPartial>), _>(
				|entity, (parents, partials)| {
					PathPattern::namespace_root(entity, &parents, &partials)
				},
			)
			.await?;
		caller
			.world()
			.entity(space)
			.exchange(self.request)
			.await
			.xok()
	}
}

impl From<Request> for RouteCall {
	fn from(request: Request) -> Self { Self { request } }
}
