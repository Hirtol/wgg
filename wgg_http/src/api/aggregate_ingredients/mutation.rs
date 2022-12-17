use async_graphql::{Context, MaybeUndefined, Object};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, IntoActiveValue, QueryFilter, TransactionTrait,
};
use wgg_providers::models::Provider;

use crate::db::EntityExt;
use crate::db::{Id, IntoActiveValueExt, SelectExt};
use crate::{
    api::{error::GraphqlError, ContextExt, GraphqlResult, ProductId, MAX_AMOUNT_DELETE},
    db,
};

use super::objects::AggregateIngredient;

#[derive(Default)]
pub struct AggregateMutation;

#[Object]
impl AggregateMutation {
    /// Create a new aggregate ingredient.
    /// The sub-ingredients list should have at least one ingredient inside.
    /// The first in the aforementioned list's image will be used as the image for the aggregate ingredient, this can later be changed.
    ///
    /// # Returns
    ///
    /// The newly created aggregate ingredient.
    ///
    /// # Accessible By
    ///
    /// Everyone.
    #[tracing::instrument(skip(self, ctx))]
    async fn aggregate_ingredient_create(
        &self,
        ctx: &Context<'_>,
        input: AggregateCreateInput,
    ) -> GraphqlResult<AggregateCreatePayload> {
        let state = ctx.wgg_state();
        let current_user = ctx.wgg_user()?;
        let product_image = if let Some(ingredient) = input.ingredients.first() {
            Some(
                state
                    .providers
                    .search_product(ingredient.provider, &ingredient.id)
                    .await?
                    .image_url,
            )
        } else {
            None
        };

        let tx = state.db.begin().await?;

        let new_aggregate = db::agg_ingredients::ActiveModel {
            created_by: current_user.id.into_active_value(),
            name: input.name.into_active_value(),
            image_url: product_image.into_active_value(),
            ..Default::default()
        };
        let model = new_aggregate.insert(&tx).await?;

        if !input.ingredients.is_empty() {
            let new_ingredients = input
                .ingredients
                .into_iter()
                .map(|item| db::agg_ingredients_links::ActiveModel {
                    id: Default::default(),
                    aggregate_id: model.id.into_active_value(),
                    provider_id: state.provider_id_from_provider(&item.provider).into_active_value(),
                    provider_ingr_id: item.id.into_active_value(),
                });
            let _ = db::agg_ingredients_links::Entity::insert_many(new_ingredients)
                .exec(&tx)
                .await?;
        }

        tx.commit().await?;

        tracing::debug!(
            new_ingredient=?model,
            "New aggregate ingredient created"
        );

        Ok(AggregateCreatePayload { data: model.into() })
    }

    /// Update an aggregate ingredient.
    ///
    /// # Returns
    ///
    /// The newly updated aggregate ingredient.
    ///
    /// # Accessible By
    ///
    /// Everyone. One can only update aggregate ingredients owned by the current viewer, unless they're an admin.
    #[tracing::instrument(skip(self, ctx))]
    async fn aggregate_ingredient_update(
        &self,
        ctx: &Context<'_>,
        id: Id,
        input: AggregateUpdateChangeSet,
    ) -> GraphqlResult<AggregateUpdatePayload> {
        let state = ctx.wgg_state();
        let current_user = ctx.wgg_user()?;

        let tx = state.db.begin().await?;

        let current_aggregate = db::agg_ingredients::Entity::find_by_id(id).one_or_err(&tx).await?;

        if !current_user.is_admin && current_user.id != current_aggregate.created_by {
            return Err(GraphqlError::Unauthorized);
        }

        tracing::debug!(current=?current_aggregate, "Updating aggregate ingredient");

        let mut update = current_aggregate.into_active_model();

        update.name = input.name.into_flattened_active_value();
        update.image_url = input.image_url.into_flattened_active_value();

        let model = update.update(&tx).await?;

        if let Some(ingredients) = input.ingredients.take() {
            // Delete existing ingredients associated with this ID
            let _ = db::agg_ingredients_links::Entity::delete_many()
                .filter(db::agg_ingredients_links::Column::AggregateId.eq(id))
                .exec(&tx)
                .await?;

            if !ingredients.is_empty() {
                // (Re)create the ingredients
                let new_ingredients = ingredients
                    .into_iter()
                    .map(|item| db::agg_ingredients_links::ActiveModel {
                        id: Default::default(),
                        aggregate_id: id.into_active_value(),
                        provider_id: state
                            .db_providers
                            .get(&item.provider)
                            .copied()
                            .into_flattened_active_value(),
                        provider_ingr_id: item.id.into_active_value(),
                    });
                let _ = db::agg_ingredients_links::Entity::insert_many(new_ingredients)
                    .exec(&tx)
                    .await?;
            }
        }

        tx.commit().await?;

        tracing::debug!(
            update_ingredient=?model,
            "Updated aggregate ingredient"
        );

        Ok(AggregateUpdatePayload { data: model.into() })
    }

    /// Delete an aggregate ingredient.
    /// All sub-ingredients referencing this ingredient will be deleted as well.
    ///
    /// # Accessible By
    ///
    /// Everyone. One can only delete aggregate ingredients owned by the current viewer, unless they're an admin.
    #[tracing::instrument(skip(self, ctx))]
    async fn aggregate_ingredient_delete(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "All aggregate ingredient ids to delete")] ids: Vec<Id>,
    ) -> GraphqlResult<AggregateDeletePayload> {
        let state = ctx.wgg_state();
        let current_user = ctx.wgg_user()?;

        if ids.len() > MAX_AMOUNT_DELETE {
            return Err(GraphqlError::InvalidInput(format!(
                "One can delete at most `{}` items at a time, not `{}`",
                MAX_AMOUNT_DELETE,
                ids.len()
            )));
        }

        let delete = db::agg_ingredients::Entity::delete_by_ids(ids.iter().copied());
        let tx = state.db.begin().await?;

        let deleted = if current_user.is_admin {
            // No need to check for user ownership
            delete.exec(&tx).await?.rows_affected
        } else {
            let result = delete
                .filter(db::agg_ingredients::created_by(current_user.id))
                .exec(&tx)
                .await?;

            if result.rows_affected < ids.len() as u64 {
                return Err(GraphqlError::InvalidInput(format!(
                    "One can only delete records belonging to the current user, \
                    wanted to delete `{}`, but could only delete `{}`",
                    ids.len(),
                    result.rows_affected
                )));
            }

            result.rows_affected
        };

        tx.commit().await?;

        tracing::debug!(deleted = deleted, "Deleted aggregate ingredients");

        Ok(AggregateDeletePayload { deleted })
    }
}

#[derive(Debug, async_graphql::InputObject)]
pub struct AggregateCreateInput {
    pub name: String,
    pub ingredients: Vec<ProviderProductInput>,
}

#[derive(Debug, async_graphql::InputObject)]
pub struct ProviderProductInput {
    pub id: ProductId,
    pub provider: Provider,
}

#[derive(Debug, async_graphql::SimpleObject)]
pub struct AggregateCreatePayload {
    /// The newly created aggregate ingredient
    pub data: AggregateIngredient,
}

#[derive(Debug, async_graphql::InputObject)]
pub struct AggregateUpdateChangeSet {
    pub name: MaybeUndefined<String>,
    pub ingredients: MaybeUndefined<Vec<ProviderProductInput>>,
    pub image_url: MaybeUndefined<String>,
}

#[derive(Debug, async_graphql::SimpleObject)]
pub struct AggregateUpdatePayload {
    /// The updated aggregate ingredient
    pub data: AggregateIngredient,
}

#[derive(Debug, async_graphql::SimpleObject)]
pub struct AggregateDeletePayload {
    /// The amount of aggregate ingredients deleted.
    pub deleted: u64,
}
