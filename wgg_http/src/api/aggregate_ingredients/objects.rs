use crate::api::error::GraphqlError;
use crate::api::{ContextExt, GraphqlResult};
use crate::{db, db::Id};
use async_graphql::{ComplexObject, Context, SimpleObject};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use wgg_providers::models::WggSearchProduct;

#[derive(Clone, Debug, SimpleObject)]
#[graphql(complex)]
pub struct AggregateIngredient {
    pub id: Id,
    pub name: String,
    pub image_url: String,
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
        let links = db::agg_ingredients_links::Entity::find()
            .filter(db::agg_ingredients_links::Column::AggregateId.eq(self.id))
            .find_also_related(db::providers::Entity)
            .all(&state.db)
            .await?;

        let futures = links.into_iter().map(|(item, provider)| async move {
            let provider = provider
                .ok_or_else(|| GraphqlError::InternalError("Could not find a provider in the database".to_string()))?
                .name
                .parse()?;

            let item = state
                .providers
                .search_product_by_id(provider, &item.provider_ingr_id)
                .await?;

            Ok::<_, GraphqlError>(item)
        });

        let results = futures::future::join_all(futures).await.into_iter().try_collect()?;

        Ok(results)
    }
}
