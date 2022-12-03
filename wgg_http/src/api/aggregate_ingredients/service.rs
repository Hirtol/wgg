use crate::api::{GraphqlResult, State};
use crate::db;
use crate::db::Id;
use sea_orm::{EntityTrait, QueryFilter, TransactionTrait};
use wgg_providers::models::CentPrice;

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
    let price_sum = products.into_iter().map(|i| i.display_price).sum();

    tx.commit().await?;

    Ok((price_sum, total_length))
}
