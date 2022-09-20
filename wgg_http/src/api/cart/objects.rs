use crate::api::auth::AuthContext;
use crate::api::error::GraphqlError;
use crate::api::{ContextExt, GraphqlResult, ProductId};
use crate::db;
use crate::db::{Id, SelectExt};
use async_graphql::{Context, SimpleObject};
use chrono::{DateTime, Utc};
use sea_orm::{EntityTrait, ModelTrait, TransactionTrait};
use wgg_providers::models::Provider;

#[derive(Clone, Debug, SimpleObject)]
#[graphql(complex)]
pub struct UserCart {
    id: Id,
    #[graphql(skip)]
    model: db::cart::Model,
}

#[async_graphql::ComplexObject]
impl UserCart {
    /// When a cart has been *resolved*, then it is marked as completed.
    pub async fn completed(&self) -> bool {
        self.model.completed_at.is_some()
    }

    /// When a cart has been *resolved*, then it is marked as completed.
    pub async fn completed_at(&self) -> Option<DateTime<Utc>> {
        self.model.completed_at
    }

    /// When a cart has been *resolved*, then a particular provider will also have been picked for that cart.
    pub async fn picked_provider(&self, ctx: &Context<'_>) -> Option<Provider> {
        let state = ctx.wgg_state();

        self.model
            .picked_id
            .and_then(|picked_id| state.provider_from_id(picked_id))
    }

    /// Return the current (possibly outdated!) price tallies for the providers relevant to this cart.
    /// One should *resolve* the current cart in order to get the most up-to-date prices.
    ///
    /// Note that the tallies include provider specific products (e.g, if you only have milk from Picnic, but not Jumbo,
    /// Picnic will have a higher tally)
    pub async fn tallies(&self, ctx: &Context<'_>) -> GraphqlResult<Vec<CartTally>> {
        let state = ctx.wgg_state();
        let result = self.model.find_related(db::cart_tally::Entity).all(&state.db).await?;

        Ok(result.into_iter().map(|tally| tally.into()).collect())
    }

    /// Return all the contents of the current cart, notes, products, and aggregates.
    pub async fn contents(&self, ctx: &Context<'_>) -> GraphqlResult<Vec<CartContent>> {
        let state = ctx.wgg_state();

        let tx = state.db.begin().await?;

        let notes = self.model.find_related(db::cart_contents::notes::Entity).all(&tx);
        let products = self.model.find_related(db::cart_contents::raw_product::Entity).all(&tx);
        let aggregate = self.model.find_related(db::cart_contents::aggregate::Entity).all(&tx);
        let (notes, products, aggregate) = futures::future::try_join3(notes, products, aggregate).await?;

        let result = std::iter::empty()
            .chain(notes.into_iter().map(|note| CartContent::Note(note.into())))
            .chain(products.into_iter().map(|product| CartContent::Product(product.into())))
            .chain(aggregate.into_iter().map(|agg| CartContent::Aggregate(agg.into())))
            .collect();

        Ok(result)
    }

    /// Return the owner of this cart.
    ///
    /// # Accessible by
    ///
    /// Everyone. If the current cart is not owned by the current user then the current user needs to be an admin.
    pub async fn owner(&self, ctx: &Context<'_>) -> GraphqlResult<AuthContext> {
        let state = ctx.wgg_state();
        let current_user = ctx.wgg_user()?;

        if current_user.id == self.model.user_id {
            Ok(current_user.clone())
        } else if current_user.is_admin {
            Ok(db::users::Entity::find_by_id(self.model.user_id)
                .one_or_err(&state.db)
                .await?
                .into())
        } else {
            Err(GraphqlError::Unauthorized)
        }
    }
}

#[derive(Clone, Debug, async_graphql::Interface)]
#[graphql(
    field(name = "id", type = "&Id"),
    field(name = "quantity", type = "&u32"),
    field(name = "created_at", type = "&DateTime<Utc>")
)]
pub enum CartContent {
    Note(CartNoteProduct),
    Product(CartProviderProduct),
    Aggregate(CartAggregateProduct),
}

#[derive(Clone, Debug, SimpleObject)]
pub struct CartNoteProduct {
    pub id: Id,
    #[graphql(skip)]
    pub cart_id: Id,
    pub note: String,
    pub quantity: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct CartProviderProduct {
    pub id: Id,
    #[graphql(skip)]
    pub cart_id: Id,
    #[graphql(skip)]
    pub provider_id: Id,
    pub provider_product_id: ProductId,
    pub quantity: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct CartAggregateProduct {
    pub id: Id,
    #[graphql(skip)]
    pub cart_id: Id,
    #[graphql(skip)]
    pub aggregate_id: Id,
    pub quantity: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct CartTally {
    model: db::cart_tally::Model,
}

#[async_graphql::Object]
impl CartTally {
    pub async fn price_cents(&self) -> u32 {
        self.model.price_cents as u32
    }

    pub async fn provider(&self, ctx: &Context<'_>) -> GraphqlResult<Provider> {
        let state = ctx.wgg_state();
        state
            .provider_from_id(self.model.provider_id)
            .ok_or(GraphqlError::ResourceNotFound)
    }
}

impl From<db::cart_contents::notes::Model> for CartNoteProduct {
    fn from(model: db::cart_contents::notes::Model) -> Self {
        Self {
            id: model.id,
            cart_id: model.cart_id,
            note: model.note,
            quantity: model.quantity as u32,
            created_at: model.created_at,
        }
    }
}
impl From<db::cart_contents::raw_product::Model> for CartProviderProduct {
    fn from(model: db::cart_contents::raw_product::Model) -> Self {
        Self {
            id: model.id,
            cart_id: model.cart_id,
            provider_id: model.provider_id,
            provider_product_id: model.provider_product,
            quantity: model.quantity as u32,
            created_at: model.created_at,
        }
    }
}
impl From<db::cart_contents::aggregate::Model> for CartAggregateProduct {
    fn from(model: db::cart_contents::aggregate::Model) -> Self {
        Self {
            id: model.id,
            cart_id: model.cart_id,
            aggregate_id: model.aggregate_id,
            quantity: model.quantity as u32,
            created_at: model.created_at,
        }
    }
}

impl From<db::cart_tally::Model> for CartTally {
    fn from(model: db::cart_tally::Model) -> Self {
        CartTally { model }
    }
}

impl From<db::cart::Model> for UserCart {
    fn from(model: db::cart::Model) -> Self {
        UserCart { id: model.id, model }
    }
}
