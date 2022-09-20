use crate::api::cart::objects::UserCart;
use crate::api::error::GraphqlError;
use crate::api::pagination::{ConnectionResult, QueryResult};
use crate::api::{ContextExt, GraphqlResult};
use crate::cross_system::Filter;
use crate::db::{Id, SelectExt};
use crate::{api, cross_system, db};
use async_graphql::Context;
use sea_orm::{EntityTrait, QueryFilter};

#[derive(Default)]
pub struct CartQuery;

#[async_graphql::Object]
impl CartQuery {
    /// Return the current (un-resolved) cart for the viewer.
    ///
    /// # Accessible By
    ///
    /// Everyone.
    #[tracing::instrument(skip(self, ctx))]
    pub async fn cart_current(&self, ctx: &Context<'_>) -> GraphqlResult<UserCart> {
        let state = ctx.wgg_state();
        let user = ctx.wgg_user()?;

        let cart = db::cart::Entity::find()
            .filter(db::cart::has_user(user.id))
            .filter(db::cart::is_completed().not())
            .one_or_err(&state.db)
            .await?;

        Ok(cart.into())
    }

    #[tracing::instrument(skip(self, ctx))]
    pub async fn carts(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        first: Option<i32>,
        #[graphql(desc = "Filters for the collection")] filters: Option<Filter<CartList>>,
    ) -> ConnectionResult<UserCart> {
        let state = ctx.wgg_state();
        let user = ctx.wgg_user()?;

        api::pagination::offset_query(after, first, |offset, limit| async move {
            let conditions = cross_system::recursive_filter(filters, |mut cond, fields| {
                if let Some(user_id) = fields.owned_by {
                    cond = cond.add(db::cart::has_user(user_id))
                }

                if let Some(is_completed) = fields.is_completed {
                    let condition = db::cart::is_completed();
                    cond = cond.add(if is_completed { condition } else { condition.not() });
                }

                cond
            })?;

            let pagination = db::cart::Entity::find()
                .filter(conditions)
                .offset_paginate(limit as u64, &state.db);
            let (result, total_count) = pagination.fetch_and_count(offset.unwrap_or_default().offset()).await?;

            // Authorization
            if !user.is_admin && result.iter().any(|item| item.user_id != user.id) {
                // Check that they're only querying for their own carts.
                Err(GraphqlError::Unauthorized)
            } else {
                Ok(QueryResult {
                    iter: result.into_iter().map(|i| i.into()),
                    total_count,
                })
            }
        })
        .await
    }
}

/// Filter fields for [UserCart] queries.
#[derive(async_graphql::InputObject, Debug, Default)]
pub struct CartList {
    /// The user id who owns a given cart.
    pub owned_by: Option<Id>,
    /// Whether the cart has been resolved (aka completed)
    pub is_completed: Option<bool>,
}
