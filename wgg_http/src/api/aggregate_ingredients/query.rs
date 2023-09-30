use crate::api::aggregate_ingredients::objects::AggregateIngredient;
use crate::api::pagination::{ConnectionResult, QueryResult};
use crate::api::{self, ContextExt, GraphqlResult, ProductId};
use crate::cross_system::{self, Filter};
use crate::db::{self};
use async_graphql::{Context, Object};
use sea_orm::{ColumnTrait, EntityTrait, IntoSimpleExpr, QueryFilter};
use wgg_db_entity::{DbId, SelectExt};
use wgg_providers::models::Provider;

#[derive(Default)]
pub struct AggregateQuery;

#[Object]
impl AggregateQuery {
    /// Returns all aggregate ingredients owned by the current user.
    ///
    /// # Accessible By
    ///
    /// Everyone.
    #[tracing::instrument(skip(self, ctx))]
    async fn aggregate_ingredients(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        first: Option<i32>,
        #[graphql(desc = "Filters for the collection")] filters: Option<Filter<IngredientQuery>>,
    ) -> ConnectionResult<AggregateIngredient> {
        let state = ctx.wgg_state();
        let user = ctx.wgg_user()?;

        api::pagination::offset_query(after, first, |offset, limit| async move {
            let conditions = cross_system::recursive_filter(filters, |mut cond, fields| {
                if let Some(name) = fields.has_name {
                    cond = cond.add(db::agg_ingredients::has_name_like(&name))
                }
                if let Some(combo) = fields.has_product {
                    use sea_orm::sea_query::*;
                    let provider_id = state.provider_id_from_provider(&combo.provider);

                    let subquery = Query::select()
                        .expr(db::agg_ingredients::Column::Id.into_simple_expr())
                        .from(db::agg_ingredients::Entity)
                        .left_join(
                            db::agg_ingredients_links::Entity,
                            Expr::col((db::agg_ingredients::Entity, db::agg_ingredients::Column::Id)).equals((
                                db::agg_ingredients_links::Entity,
                                db::agg_ingredients_links::Column::AggregateId,
                            )),
                        )
                        .cond_where(db::agg_ingredients_links::related_product(
                            &combo.product_id,
                            provider_id,
                        ))
                        .to_owned();

                    cond = cond.add(db::agg_ingredients::Column::Id.in_subquery(subquery))
                }

                cond
            })?
            .add(db::agg_ingredients::created_by(user.id));

            let pagination = db::agg_ingredients::Entity::find()
                .filter(conditions)
                .offset_paginate(limit as u64, &state.db);

            let (result, total_count) = pagination.fetch_and_count(offset.unwrap_or_default().offset()).await?;

            Ok(QueryResult {
                iter: result.into_iter().map(|i| i.into()),
                total_count,
            })
        })
        .await
    }

    /// Returns the specific aggregate ingredient, if it is owned by the current user and exists.
    ///
    /// # Accessible By
    ///
    /// Everyone.
    #[tracing::instrument(skip(self, ctx))]
    async fn aggregate_ingredient(&self, ctx: &Context<'_>, id: DbId) -> GraphqlResult<AggregateIngredient> {
        let state = ctx.wgg_state();
        let user = ctx.wgg_user()?;
        Ok(db::agg_ingredients::Entity::find_by_id(id)
            .filter(db::agg_ingredients::created_by(user.id))
            .one_or_err(&state.db)
            .await?
            .into())
    }
}

#[derive(async_graphql::InputObject, Debug, Default)]
pub struct IngredientQuery {
    /// Return all aggregate ingredients which share (part) of the given name
    has_name: Option<String>,
    /// Return all aggregate ingredients which have the following product id as part of their ingredients
    has_product: Option<ProductProvider>,
}

#[derive(async_graphql::InputObject, Debug)]
pub struct ProductProvider {
    product_id: ProductId,
    provider: Provider,
}
