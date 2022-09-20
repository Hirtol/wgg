use crate::api::auth::AuthContext;
use crate::api::{ContextExt, GraphqlResult};
use async_graphql::{Context, Object};

#[derive(Default)]
pub struct AuthQuery;

#[Object]
impl AuthQuery {
    /// Returns the current user
    async fn viewer<'a>(&self, ctx: &'a Context<'a>) -> GraphqlResult<&'a AuthContext> {
        ctx.wgg_user()
    }
}
