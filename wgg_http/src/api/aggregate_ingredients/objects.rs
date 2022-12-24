use crate::api::providers::WggSearchProductWrapper;
use crate::api::{ContextExt, GraphqlResult};
use crate::{db, db::Id};
use async_graphql::{ComplexObject, Context, SimpleObject};
use chrono::{DateTime, Utc};
use sea_orm::{EntityTrait, QueryFilter};
use wgg_providers::models::CentPrice;

#[derive(Clone, Debug, SimpleObject)]
#[graphql(complex)]
pub struct AggregateIngredient {
    pub id: Id,
    pub name: String,
    pub image_url: Option<String>,
    #[graphql(skip)]
    pub created_by: Id,
    pub created_at: DateTime<Utc>,
}

#[ComplexObject]
impl AggregateIngredient {
    /// Return all composite ingredients which are part of this aggregate ingredient.
    #[tracing::instrument(skip(ctx))]
    pub async fn ingredients(&self, ctx: &Context<'_>) -> GraphqlResult<Vec<WggSearchProductWrapper>> {
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

    /// Returns the average price of all constituent ingredients.
    pub async fn price(&self, ctx: &Context<'_>, format: PriceFilter) -> GraphqlResult<CentPrice> {
        let products = self.ingredients(ctx).await??;
        let price_iter = products.iter().map(|i| i.item.price_info.display_price);

        let result = match format {
            PriceFilter::Minimum => price_iter.min().unwrap_or_default(),
            PriceFilter::Average => price_iter.sum::<CentPrice>() / products.len().max(1) as CentPrice,
            PriceFilter::Maximum => price_iter.max().unwrap_or_default(),
        };

        Ok(result)
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
        }
    }
}

#[derive(Debug, Clone, Copy, async_graphql::Enum, PartialEq, Eq)]
pub enum PriceFilter {
    Minimum,
    Average,
    Maximum,
}
