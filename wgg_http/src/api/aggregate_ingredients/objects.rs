use crate::api::{ContextExt, GraphqlResult};
use crate::{db, db::Id};
use async_graphql::{ComplexObject, Context, SimpleObject};
use chrono::{DateTime, Utc};
use sea_orm::{EntityTrait, QueryFilter};
use wgg_providers::models::WggSearchProduct;

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
    pub async fn ingredients(&self, ctx: &Context<'_>) -> GraphqlResult<Vec<WggSearchProduct>> {
        let state = ctx.wgg_state();

        let products = db::agg_ingredients_links::Entity::find()
            .filter(db::agg_ingredients_links::related_aggregate(self.id))
            .all(&state.db)
            .await?;

        let product_futures = products.into_iter().map(|model| {
            let provider = state.provider_from_id(model.provider_id);
            state.providers.search_product_by_id(provider, model.provider_ingr_id)
        });

        let results = futures::future::try_join_all(product_futures).await?;

        Ok(results)
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
