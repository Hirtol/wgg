use crate::api::auth::AuthContext;
use crate::api::error::GraphqlError;
use crate::api::{GraphqlResult, State};
use async_graphql::Context;

pub(crate) trait ContextExt {
    /// Retrieve the [`State`] from the context
    fn wgg_state(&self) -> &State;

    /// Retrieve the current [AuthContext] from the request.
    ///
    /// If the current request had no authenticated user associated with it, then this returns a [Graphql::Unauthorized]
    fn wgg_user(&self) -> GraphqlResult<&AuthContext>;
}

impl<'a> ContextExt for Context<'a> {
    fn wgg_state(&self) -> &State {
        self.data_unchecked()
    }

    fn wgg_user(&self) -> GraphqlResult<&'a AuthContext> {
        self.data().map_err(|_| GraphqlError::Unauthorized)
    }
}
