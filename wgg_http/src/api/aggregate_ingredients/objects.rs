use crate::api::providers::WggSearchProductWrapper;
use crate::api::{ContextExt, GraphqlResult};
use crate::{db, db::Id};
use async_graphql::{ComplexObject, Context, SimpleObject};
use chrono::{DateTime, Utc};
use sea_orm::{EntityTrait, QueryFilter};
use wgg_providers::models::CentPrice;

/// An aggregate ingredient is a collection of concrete, provider specific, products.
///
/// This addresses the problem of name-matching, by allowing the user to define product categories themselves.
/// For example, one might define an 'Eggs' aggregate ingredient. This is then composed of one egg product from 'Jumbo',
/// and another from 'Picnic'. On sale resolution the prices of these respective providers are used for the final calculation.
///
/// When an aggregate ingredient is in the cart it will always *resolve* to at most one concrete product for each specific provider.
///
/// This does not mean one ingredient can't have multiple products of a single provider, just that the sale resolution would only pick one of them.
/// TODO: Implement this sale resolution (Note, depends on more fine grained quantities within aggregate products).
#[derive(Clone, Debug, SimpleObject)]
#[graphql(complex)]
pub struct AggregateIngredient {
    pub id: Id,
    pub name: String,
    pub image_url: Option<String>,
    #[graphql(skip)]
    pub created_by: Id,
    pub created_at: DateTime<Utc>,
    /// Lazily initialised as it is shared between multiple resolvers
    #[graphql(skip)]
    pub ingredients: tokio::sync::OnceCell<Vec<WggSearchProductWrapper>>,
}

#[ComplexObject]
impl AggregateIngredient {
    /// Return all composite ingredients which are part of this aggregate ingredient.
    #[tracing::instrument(skip(ctx))]
    pub async fn ingredients(&self, ctx: &Context<'_>) -> GraphqlResult<&Vec<WggSearchProductWrapper>> {
        self.get_ingredients(ctx).await
    }

    /// Returns the average price of all constituent ingredients.
    #[tracing::instrument(skip(self, ctx))]
    pub async fn price(&self, ctx: &Context<'_>, format: PriceFilter) -> GraphqlResult<CentPrice> {
        let products = self.get_ingredients(ctx).await?;
        let price_iter = products.iter().map(|i| i.item.price_info.display_price);

        let result = match format {
            PriceFilter::Minimum => price_iter.min().unwrap_or_default(),
            PriceFilter::Average => price_iter.sum::<CentPrice>() / products.len().max(1) as CentPrice,
            PriceFilter::Maximum => price_iter.max().unwrap_or_default(),
        };

        Ok(result)
    }

    /// Retrieve the direct quantity of this product within the given `cart_id`.
    ///
    /// If `cart_id` is not given then the current cart of the user is assumed.
    #[tracing::instrument(skip(self, ctx))]
    pub async fn direct_quantity(&self, ctx: &Context<'_>, cart_id: Option<Id>) -> GraphqlResult<Option<u32>> {
        let state = ctx.wgg_state();
        let user = ctx.wgg_user()?;
        crate::api::cart::get_aggregate_product_quantity(&state.db, cart_id, user.id, self.id).await
    }
}

impl AggregateIngredient {
    async fn get_ingredients(&self, ctx: &Context<'_>) -> GraphqlResult<&Vec<WggSearchProductWrapper>> {
        self.ingredients
            .get_or_try_init(move || self.get_ingredients_initialisation(ctx))
            .await
    }

    async fn get_ingredients_initialisation(&self, ctx: &Context<'_>) -> GraphqlResult<Vec<WggSearchProductWrapper>> {
        let state = ctx.wgg_state();

        let products = db::agg_ingredients_links::Entity::find()
            .filter(db::agg_ingredients_links::related_aggregate(self.id))
            .all(&state.db)
            .await?;

        let product_futures = products.into_iter().map(|model| {
            let provider = state.provider_from_id(model.provider_id);
            state.providers.search_product(provider, model.provider_ingr_id)
        });

        let results = futures::future::try_join_all(product_futures).await?;

        Ok(results.into_iter().map(|i| i.into()).collect())
    }
}

impl From<db::agg_ingredients::Model> for AggregateIngredient {
    fn from(model: db::agg_ingredients::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            image_url: model.image_url,
            created_by: model.created_by,
            created_at: model.created_at,
            ingredients: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, async_graphql::Enum, PartialEq, Eq)]
pub enum PriceFilter {
    Minimum,
    Average,
    Maximum,
}
