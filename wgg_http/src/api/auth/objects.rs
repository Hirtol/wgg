use crate::api::cart::{CartFilterFields, UserCart};
use crate::api::error::GraphqlError;
use crate::api::pagination::ConnectionResult;
use crate::api::{ContextExt, GraphqlResult};
use crate::cross_system::Filter;
use crate::{api, db};
use async_graphql::Context;
use wgg_db_entity::DbId;

/// Represents a user that is already logged in.
/// Implements [axum::extract::FromRequest] and can therefore be requested in HTTP service methods.
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct AuthContext {
    pub id: DbId,
    pub email: String,
    pub username: String,
    pub is_admin: bool,
}

#[async_graphql::ComplexObject]
impl AuthContext {
    /// Return the current cart in use by this user
    #[tracing::instrument(skip(self, ctx))]
    pub async fn current_cart(&self, ctx: &Context<'_>) -> GraphqlResult<UserCart> {
        let user = ctx.wgg_user()?;

        // In theory a regular user shouldn't be able to acquire an object from other users to refer to this resolver,
        // but just to be safe...
        if user.is_admin || user.id == self.id {
            let mut filters = Filter::new(CartFilterFields::default());
            filters.fields.owned_by = Some(self.id);
            filters.fields.is_completed = Some(false);

            Ok(api::cart::CartQuery
                .carts(ctx, None, Some(1), Some(filters))
                .await??
                .edges
                .pop()
                .ok_or(GraphqlError::ResourceNotFound)?
                .node)
        } else {
            Err(GraphqlError::Unauthorized)
        }
    }

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
