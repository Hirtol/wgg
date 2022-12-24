use crate::api::{GraphqlResult, State};
use crate::db;
use crate::db::Id;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, TransactionTrait};
use wgg_providers::models::CentPrice;
use crate::api::aggregate_ingredients::AggregateIngredient;

/// Perform a reverse-lookup for all aggregate ingredients associated with the given `product_id`.
pub async fn get_associated_aggregate_for_product(db: &impl ConnectionTrait, user_id: Id, provider_id: Id, product_id: &str) -> GraphqlResult<Vec<AggregateIngredient>> {
    let aggregate = db::agg_ingredients::Entity::find()
        .filter(db::agg_ingredients::created_by(user_id))
        .left_join(db::agg_ingredients_links::Entity)
        .filter(db::agg_ingredients_links::Column::ProviderId.eq(provider_id))
        .filter(db::agg_ingredients_links::Column::ProviderIngrId.eq(product_id))
        .all(db).await?;
    
    Ok(aggregate.into_iter().map(|i| i.into()).collect())
}

/// Calculate the total price of the given aggregate ingredient.
///
/// # Returns
///
/// The cent price, alongside the total amount of items that are part of the given ingredient.
#[tracing::instrument(skip(db, state))]
#[allow(dead_code)]
pub async fn calculate_aggregate_total_price(
    db: &impl TransactionTrait,
    state: &State,
    aggregate_id: Id,
) -> GraphqlResult<(CentPrice, usize)> {
    let tx = db.begin().await?;

    let aggregate = db::agg_ingredients_links::Entity::find()
        .filter(db::agg_ingredients_links::related_aggregate(aggregate_id))
        .all(&tx)
        .await?;

    let products = futures::future::try_join_all(aggregate.into_iter().map(|ingr| {
        let provider = state.provider_from_id(ingr.provider_id);
        state.providers.search_product(provider, ingr.provider_ingr_id)
    }))
    .await?;

    let total_length = products.len();
    let price_sum = products.into_iter().map(|i| i.price_info.display_price).sum();

    tx.commit().await?;

    Ok((price_sum, total_length))
}
