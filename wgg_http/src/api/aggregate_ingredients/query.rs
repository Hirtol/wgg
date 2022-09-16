use crate::api::aggregate_ingredients::objects::AggregateIngredient;
use crate::api::pagination::{ConnectionResult, QueryResult};
use crate::api::ContextExt;
use crate::db::{self, SelectExt};
use async_graphql::{Context, Object};
use sea_orm::{Condition, EntityTrait, QueryFilter};

#[derive(Default)]
pub struct AggregateQuery;

#[Object]
impl AggregateQuery {
    /// Returns all aggregate ingredients in the back-end
    #[tracing::instrument(skip(self, ctx))]
    async fn agg_ingredients(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        first: Option<i32>,
        #[graphql(desc = "Filters for the collection")] filters: Option<IngredientQueryFilter>,
    ) -> ConnectionResult<AggregateIngredient> {
        let state = ctx.wgg_state();

        crate::api::pagination::offset_query(after, first, |offset, limit| async move {
            let mut conditions = Condition::all();

            if let Some(filters) = filters {
                if let Some(name) = filters.has_name {
                    conditions = conditions.add(db::agg_ingredients::has_name_like(&name));
                }
            }

            let pagination = db::agg_ingredients::Entity::find()
                .filter(conditions)
                .offset_paginate(limit as u64, &state.db);

            let result = pagination
                .fetch_offset(offset.unwrap_or_default().offset())
                .await?
                .into_iter()
                .map(|i| AggregateIngredient {
                    id: i.id,
                    name: i.name,
                    image_url: i.image_url,
                    created_by: i.created_by,
                    created_at: i.created_at,
                });

            Ok(QueryResult {
                iter: result,
                total_count: Some(pagination.num_items().await?),
            })
        })
        .await
    }
}

#[derive(async_graphql::InputObject, Debug, Default)]
pub struct IngredientQueryFilter {
    /// Return all aggregate ingredients which share (part) of the given name
    has_name: Option<String>,
}
