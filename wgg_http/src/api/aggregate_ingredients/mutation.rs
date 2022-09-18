use async_graphql::{Context, Object};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveValue, TransactionTrait};
use wgg_providers::models::Provider;

use crate::db::IntoActiveValueExt;
use crate::{
    api::{error::GraphqlError, ContextExt, GraphqlResult, ProductId},
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
            state
                .providers
                .search_product_by_id(ingredient.provider, &ingredient.id)
                .await?
                .image_url
        } else {
            return Err(GraphqlError::InvalidInput(
                "Need at least one sub-ingredient to create an aggregate ingredient".to_string(),
            ));
        };

        let tx = state.db.begin().await?;

        let new_aggregate = db::agg_ingredients::ActiveModel {
            created_by: current_user.id.into_active_value(),
            name: input.name.into_active_value(),
            image_url: product_image.into_active_value(),
            ..Default::default()
        };
        let model = new_aggregate.insert(&tx).await?;

        let new_ingredients = input
            .ingredients
            .into_iter()
            .map(|item| db::agg_ingredients_links::ActiveModel {
                id: Default::default(),
                aggregate_id: model.id.into_active_value(),
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

        tx.commit().await?;

        tracing::debug!(
            new_ingredient=?model,
            "New aggregate ingredient created"
        );

        Ok(AggregateCreatePayload { data: model.into() })
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
