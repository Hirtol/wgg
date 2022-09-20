use crate::api::cart::{CartFilterFields, UserCart};
use crate::api::pagination::ConnectionResult;
use crate::cross_system::Filter;
use crate::db::Id;
use crate::{api, db};
use async_graphql::Context;

/// Represents a user that is already logged in.
/// Implements [axum::extract::FromRequest] and can therefore be requested in HTTP service methods.
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct AuthContext {
    pub id: Id,
    pub email: String,
    pub username: String,
    pub is_admin: bool,
}

#[async_graphql::ComplexObject]
impl AuthContext {
    /// Return all carts owned by the given user
    #[tracing::instrument(skip(self, ctx))]
    pub async fn carts(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        first: Option<i32>,
        #[graphql(desc = "Filters for the collection")] filters: Option<Filter<CartFilterFields>>,
    ) -> ConnectionResult<UserCart> {
        let mut filters = filters.unwrap_or_default();
        filters.fields.owned_by = Some(self.id);

        api::cart::CartQuery.carts(ctx, after, first, Some(filters)).await?
    }
}

impl From<db::users::Model> for AuthContext {
    fn from(model: db::users::Model) -> Self {
        AuthContext {
            id: model.id,
            email: model.email,
            username: model.username,
            is_admin: model.is_admin,
        }
    }
}
