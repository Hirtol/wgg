use crate::api::cart::UserCart;
use crate::api::error::GraphqlError;
use crate::api::{ContextExt, GraphqlResult, ProductId};
use crate::db;
use crate::db::{Id, IntoActiveValueExt};
use async_graphql::Context;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, IntoActiveValue, QueryFilter, TransactionTrait,
};
use wgg_providers::models::Provider;

#[derive(Default)]
pub struct CartMutation;

#[async_graphql::Object]
impl CartMutation {
    /// Add the provided products to the current cart.
    ///
    /// If one adds an item that is already in the cart then the count is set to the provided amount.
    ///
    /// # Accessible By
    ///
    /// Everyone.
    #[tracing::instrument(skip(self, ctx))]
    pub async fn cart_current_set_product(
        &self,
        ctx: &Context<'_>,
        input: CartAddProductInput,
    ) -> GraphqlResult<CartAddProductPayload> {
        let state = ctx.wgg_state();
        let user = ctx.wgg_user()?;

        let tx = state.db.begin().await?;
        let cart = db::cart::get_active_cart_for_user(user.id, &tx).await?;

        if let Some(note) = input.notes {
            use db::cart_contents::notes::*;
            if note.quantity == 0 {
                return Err(GraphqlError::InvalidInput(
                    "Can't set a product's quantity to 0, use remove() instead".to_string(),
                ));
            }

            let to_insert = ActiveModel {
                id: note.id.into_flattened_active_value(),
                cart_id: cart.id.into_active_value(),
                note: note.content.into_active_value(),
                quantity: (note.quantity as i32).into_active_value(),
                created_at: ActiveValue::NotSet,
            };

            let _ = to_insert.save(&tx).await?;
        }
        if let Some(raw) = input.raw_product {
            use db::cart_contents::raw_product::*;
            if raw.quantity == 0 {
                return Err(GraphqlError::InvalidInput(
                    "Can't set a product's quantity to 0, use remove() instead".to_string(),
                ));
            }

            // Insert new products.
            let to_insert = ActiveModel {
                id: ActiveValue::NotSet,
                cart_id: cart.id.into_active_value(),
                provider_id: state.provider_id_from_provider(&raw.provider).into_active_value(),
                provider_product: raw.product_id.into_active_value(),
                quantity: (raw.quantity as i32).into_active_value(),
                created_at: ActiveValue::NotSet,
            };

            let _ = Entity::insert(to_insert)
                .on_conflict(
                    OnConflict::columns([Column::CartId, Column::ProviderId, Column::ProviderProduct])
                        .update_columns([Column::Quantity])
                        .to_owned(),
                )
                .exec(&tx)
                .await?;
        }
        if let Some(aggregate) = input.aggregate {
            use db::cart_contents::aggregate::*;
            if aggregate.quantity == 0 {
                return Err(GraphqlError::InvalidInput(
                    "Can't set a product's quantity to 0, use remove() instead".to_string(),
                ));
            }

            // Insert aggregate
            let to_insert = ActiveModel {
                id: ActiveValue::NotSet,
                cart_id: cart.id.into_active_value(),
                aggregate_id: aggregate.aggregate_id.into_active_value(),
                quantity: (aggregate.quantity as i32).into_active_value(),
                created_at: ActiveValue::NotSet,
            };

            let _ = Entity::insert(to_insert)
                .on_conflict(
                    OnConflict::columns([Column::CartId, Column::AggregateId])
                        .update_columns([Column::Quantity])
                        .to_owned(),
                )
                .exec(&tx)
                .await?;
        }

        tx.commit().await?;

        Ok(CartAddProductPayload { data: cart.into() })
    }

    /// Remove the provided item from the current cart.
    ///
    /// # Accessible By
    ///
    /// Everyone.
    #[tracing::instrument(skip(self, ctx))]
    pub async fn cart_current_remove_product(
        &self,
        ctx: &Context<'_>,
        input: CartRemoveProductInput,
    ) -> GraphqlResult<CartRemoveProductPayload> {
        let state = ctx.wgg_state();
        let user = ctx.wgg_user()?;

        let tx = state.db.begin().await?;
        let cart = db::cart::get_active_cart_for_user(user.id, &tx).await?;

        if let Some(note_id) = input.notes {
            use db::cart_contents::notes::*;
            Entity::delete_by_id(note_id)
                .filter(Column::CartId.eq(cart.id))
                .exec(&tx)
                .await?;
        }
        if let Some(raw_id) = input.raw_product {
            use db::cart_contents::raw_product::*;

            Entity::delete_many()
                .filter(Column::CartId.eq(cart.id))
                .filter(Column::ProviderProduct.eq(raw_id.product_id))
                .filter(Column::ProviderId.eq(state.provider_id_from_provider(&raw_id.provider)))
                .exec(&tx)
                .await?;
        }
        if let Some(aggregate_id) = input.aggregate {
            use db::cart_contents::aggregate::*;
            Entity::delete_many()
                .filter(Column::CartId.eq(cart.id))
                .filter(Column::AggregateId.eq(aggregate_id))
                .exec(&tx)
                .await?;
        }

        tx.commit().await?;

        Ok(CartRemoveProductPayload { data: cart.into() })
    }

    /// Mark the current cart as completed, and create a new one.
    ///
    /// # Accessible By
    ///
    /// Everyone.
    #[tracing::instrument(skip(self, ctx))]
    pub async fn cart_current_complete(
        &self,
        ctx: &Context<'_>,
        input: CartCompleteInput,
    ) -> GraphqlResult<CartCompletePayload> {
        let state = ctx.wgg_state();
        let user = ctx.wgg_user()?;
        let tx = state.db.begin().await?;

        // Mark the cart as completed
        let cart = db::cart::get_active_cart_for_user(user.id, &tx).await?;

        let active_model = db::cart::ActiveModel {
            id: ActiveValue::Set(cart.id),
            completed_at: ActiveValue::Set(Some(chrono::offset::Utc::now())),
            picked_id: ActiveValue::Set(Some(state.provider_id_from_provider(&input.picked_provider))),
            ..Default::default()
        };
        // By doing this update the database triggers will create a new cart.
        let cart = active_model.update(&tx).await?;

        // Save the historic tallies
        let tallies = super::service::calculate_tallies(&state.db, cart.id, state).await?;

        if !tallies.is_empty() {
            let to_submit = tallies
                .into_iter()
                .map(|(provider, price)| db::cart_tally::ActiveModel {
                    cart_id: cart.id.into_active_value(),
                    provider_id: state.provider_id_from_provider(&provider).into_active_value(),
                    price_cents: (price as i32).into_active_value(),
                });

            let _ = db::cart_tally::Entity::insert_many(to_submit).exec(&tx).await?;
        }

        tx.commit().await?;

        Ok(CartCompletePayload { data: cart.into() })
    }
}

#[derive(Debug, async_graphql::InputObject)]
pub struct CartAddProductInput {
    pub notes: Option<NoteProductInput>,
    pub raw_product: Option<RawProductInput>,
    pub aggregate: Option<AggregateProductInput>,
}

#[derive(Debug, async_graphql::InputObject)]
pub struct CartRemoveProductInput {
    /// The note id.
    pub notes: Option<Id>,
    /// The provider product id used to add this product
    pub raw_product: Option<RemoveRawProductInput>,
    /// The aggregate id.
    pub aggregate: Option<Id>,
}

#[derive(Debug, async_graphql::InputObject)]
pub struct RemoveRawProductInput {
    pub product_id: ProductId,
    pub provider: Provider,
}

#[derive(Debug, async_graphql::InputObject)]
pub struct NoteProductInput {
    /// If the note already exists and you want to update it then set this Id,
    /// otherwise a new note will be created based on `content`.
    pub id: Option<Id>,
    pub content: String,
    pub quantity: u32,
}

#[derive(Debug, async_graphql::InputObject)]
pub struct RawProductInput {
    pub product_id: ProductId,
    pub provider: Provider,
    pub quantity: u32,
}

#[derive(Debug, async_graphql::InputObject)]
pub struct AggregateProductInput {
    pub aggregate_id: Id,
    pub quantity: u32,
}

#[derive(Debug, async_graphql::InputObject)]
pub struct CartCompleteInput {
    pub picked_provider: Provider,
}

#[derive(Debug, async_graphql::SimpleObject)]
pub struct CartAddProductPayload {
    /// The current cart
    pub data: UserCart,
}

#[derive(Debug, async_graphql::SimpleObject)]
pub struct CartRemoveProductPayload {
    /// The current cart
    pub data: UserCart,
}

#[derive(Debug, async_graphql::SimpleObject)]
pub struct CartCompletePayload {
    /// The completed cart
    pub data: UserCart,
}
