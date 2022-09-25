use crate::api::aggregate_ingredients::AggregateIngredient;
use crate::api::auth::AuthContext;
use crate::api::error::GraphqlError;
use crate::api::{ContextExt, GraphqlResult, ProductId};
use crate::db;
use crate::db::{Id, SelectExt};
use async_graphql::{Context, SimpleObject};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use sea_orm::{EntityTrait, ModelTrait, TransactionTrait};
use wgg_providers::models::{CentPrice, Provider, WggSearchProduct};

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

        self.model.picked_id.map(|picked_id| state.provider_from_id(picked_id))
    }

    /// Return the current (possibly outdated!) price tallies for the providers relevant to this cart.
    /// One should *resolve* the current cart in order to get the most up-to-date prices.
    ///
    /// Note that the tallies include provider specific products (e.g, if you only have milk from Picnic, but not Jumbo,
    /// Picnic will have a higher tally)
    pub async fn tallies(&self, ctx: &Context<'_>) -> GraphqlResult<Vec<CartTally>> {
        let state = ctx.wgg_state();

        // If the cart was completed we wish to look for historic tally counts when we completed it.
        if self.model.completed_at.is_some() {
            let result = self.model.find_related(db::cart_tally::Entity).all(&state.db).await?;

            Ok(result.into_iter().map(|tally| tally.into()).collect())
        } else {
            // Otherwise we calculate the current values.
            let tallies = super::service::calculate_tallies(&state.db, self.id, state).await?;

            Ok(tallies
                .into_iter()
                .map(|(provider, price)| CartTally::Current { provider, price })
                .collect())
        }
    }

    /// Return all the contents of the current cart, notes, products, and aggregates.
    ///
    /// The contents are sorted by the timestamp they were added (recent on top)
    pub async fn contents(&self, ctx: &Context<'_>) -> GraphqlResult<Vec<CartContent>> {
        let state = ctx.wgg_state();

        let tx = state.db.begin().await?;

        let notes = self.model.find_related(db::cart_contents::notes::Entity).all(&tx);
        let products = self.model.find_related(db::cart_contents::raw_product::Entity).all(&tx);
        let aggregate = self
            .model
            .find_related(db::cart_contents::aggregate::Entity)
            .find_also_related(db::agg_ingredients::Entity)
            .all(&tx);
        let (notes, products, aggregate) = futures::future::try_join3(notes, products, aggregate).await?;

        let result = std::iter::empty()
            .chain(notes.into_iter().map(|note| CartContent::Note(note.into())))
            .chain(products.into_iter().map(|product| CartContent::Product(product.into())))
            .chain(
                aggregate
                    .into_iter()
                    .map(|item| (item.0, item.1.unwrap()))
                    .map(|agg| CartContent::Aggregate(agg.into())),
            )
            .sorted_by(|item1, item2| item1.get_created_at().cmp(item2.get_created_at()).reverse())
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

impl CartContent {
    pub fn get_created_at(&self) -> &DateTime<Utc> {
        match self {
            CartContent::Note(note) => &note.created_at,
            CartContent::Product(prod) => &prod.created_at,
            CartContent::Aggregate(agg) => &agg.created_at,
        }
    }
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
#[graphql(complex)]
pub struct CartProviderProduct {
    pub id: Id,
    #[graphql(skip)]
    pub cart_id: Id,
    #[graphql(skip)]
    pub provider_id: Id,
    #[graphql(skip)]
    pub provider_product_id: ProductId,
    pub quantity: u32,
    pub created_at: DateTime<Utc>,
}

#[async_graphql::ComplexObject]
impl CartProviderProduct {
    /// Return the product associated with this entry
    ///
    /// # Accessible by
    ///
    /// Everyone.
    pub async fn product(&self, ctx: &Context<'_>) -> GraphqlResult<WggSearchProduct> {
        let state = ctx.wgg_state();
        let provider = state.provider_from_id(self.provider_id);

        Ok(state
            .providers
            .search_product_by_id(provider, &self.provider_product_id)
            .await?)
    }
}

#[derive(Clone, Debug, SimpleObject)]
#[graphql(complex)]
pub struct CartAggregateProduct {
    pub id: Id,
    #[graphql(skip)]
    pub cart_id: Id,
    #[graphql(skip)]
    pub aggregate_model: db::agg_ingredients::Model,
    pub quantity: u32,
    pub created_at: DateTime<Utc>,
}

#[async_graphql::ComplexObject]
impl CartAggregateProduct {
    /// Return the primary aggregate product associated with this entry
    ///
    /// # Accessible by
    ///
    /// Everyone.
    pub async fn aggregate(&self, _ctx: &Context<'_>) -> GraphqlResult<AggregateIngredient> {
        Ok(self.aggregate_model.clone().into())
    }
}

#[derive(Clone, Debug)]
pub enum CartTally {
    Historical(db::cart_tally::Model),
    Current { provider: Provider, price: CentPrice },
}

#[async_graphql::Object]
impl CartTally {
    pub async fn price_cents(&self) -> u32 {
        match self {
            CartTally::Historical(model) => model.price_cents as u32,
            CartTally::Current { price, .. } => *price,
        }
    }

    pub async fn provider(&self, ctx: &Context<'_>) -> Provider {
        match self {
            CartTally::Historical(model) => {
                let state = ctx.wgg_state();

                state.provider_from_id(model.provider_id)
            }
            CartTally::Current { provider, .. } => *provider,
        }
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
impl From<(db::cart_contents::aggregate::Model, db::agg_ingredients::Model)> for CartAggregateProduct {
    fn from((model, agg): (db::cart_contents::aggregate::Model, db::agg_ingredients::Model)) -> Self {
        Self {
            id: model.id,
            cart_id: model.cart_id,
            aggregate_model: agg,
            quantity: model.quantity as u32,
            created_at: model.created_at,
        }
    }
}

impl From<db::cart_tally::Model> for CartTally {
    fn from(model: db::cart_tally::Model) -> Self {
        CartTally::Historical(model)
    }
}

impl From<db::cart::Model> for UserCart {
    fn from(model: db::cart::Model) -> Self {
        UserCart { id: model.id, model }
    }
}
