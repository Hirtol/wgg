use crate::api::auth::{AuthContext, WggCookies};
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

    /// Retrieve a [WggCookies] implementation
    ///
    /// Only really useful for login/logout.
    fn wgg_cookies(&self) -> WggCookies;
}

impl<'a> ContextExt for Context<'a> {
    fn wgg_state(&self) -> &State {
        self.data_unchecked()
    }

    fn wgg_user(&self) -> GraphqlResult<&'a AuthContext> {
        let auth: &'a Option<AuthContext> = self.data_unchecked();
        auth.as_ref().ok_or(GraphqlError::Unauthorized)
    }

    fn wgg_cookies(&self) -> WggCookies {
        let cookies: &tower_cookies::Cookies = self.data_unchecked();
        let key: &tower_cookies::Key = self.data_unchecked();

        WggCookies::from_cookies(cookies, key)
    }
}
