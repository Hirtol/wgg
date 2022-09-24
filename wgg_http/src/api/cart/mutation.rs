use crate::api::cart::UserCart;
use crate::api::{ContextExt, GraphqlResult, ProductId};
use crate::db;
use crate::db::{Id, IntoActiveValueExt};
use async_graphql::Context;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, EntityTrait, IntoActiveValue, TransactionTrait};
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
    pub async fn cart_current_add_product(
        &self,
        ctx: &Context<'_>,
        input: CartAddProductInput,
    ) -> GraphqlResult<CartAddProductPayload> {
        let state = ctx.wgg_state();
        let user = ctx.wgg_user()?;

        let tx = state.db.begin().await?;
        let cart = db::cart::get_active_cart_for_user(user.id, &tx).await?;

        if let Some(notes) = input.notes {
            use db::cart_contents::notes::*;
            let to_insert = notes.into_iter().map(|note| ActiveModel {
                id: ActiveValue::NotSet,
                cart_id: cart.id.into_active_value(),
                note: note.content.into_active_value(),
                quantity: (note.quantity as i32).into_active_value(),
                created_at: ActiveValue::NotSet,
            });

            let _ = Entity::insert_many(to_insert).exec(&tx).await?;
        }
        if let Some(raw) = input.raw_product {
            use db::cart_contents::raw_product::*;
            let to_insert = raw.into_iter().map(|product| ActiveModel {
                id: ActiveValue::NotSet,
                cart_id: cart.id.into_active_value(),
                provider_id: state
                    .provider_id_from_provider(&product.provider)
                    .into_flattened_active_value(),
                provider_product: product.product_id.into_active_value(),
                quantity: (product.quantity as i32).into_active_value(),
                created_at: ActiveValue::NotSet,
            });

            let _ = Entity::insert_many(to_insert)
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
            let to_insert = aggregate.into_iter().map(|agg| ActiveModel {
                id: ActiveValue::NotSet,
                cart_id: cart.id.into_active_value(),
                aggregate_id: agg.aggregate_id.into_active_value(),
                quantity: (agg.quantity as i32).into_active_value(),
                created_at: ActiveValue::NotSet,
            });

            let _ = Entity::insert_many(to_insert)
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
}

#[derive(Debug, async_graphql::InputObject)]
pub struct CartAddProductInput {
    pub notes: Option<Vec<NoteProductInput>>,
    pub raw_product: Option<Vec<RawProductInput>>,
    pub aggregate: Option<Vec<AggregateProductInput>>,
}

#[derive(Debug, async_graphql::InputObject)]
pub struct NoteProductInput {
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

#[derive(Debug, async_graphql::SimpleObject)]
pub struct CartAddProductPayload {
    /// The current cart
    pub data: UserCart,
}
