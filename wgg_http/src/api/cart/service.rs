use crate::api::{GraphqlResult, State};
use crate::db;
use crate::db::Id;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use std::collections::HashMap;
use wgg_providers::models::{CentPrice, Provider};

/// Calculate the total tally of the given cart for all providers that are part of that cart.
#[tracing::instrument(skip(db, state))]
pub async fn calculate_tallies(
    db: &impl TransactionTrait,
    cart_id: Id,
    state: &State,
) -> GraphqlResult<HashMap<Provider, CentPrice>> {
    let tx = db.begin().await?;
    let mut result = HashMap::with_capacity(state.db_providers.len());

    let products = db::cart_contents::raw_product::Entity::find()
        .filter(db::cart_contents::raw_product::Column::CartId.eq(cart_id))
        .all(&tx);

    let aggregate = db::cart_contents::aggregate::Entity::find()
        .find_with_related(db::agg_ingredients_links::Entity)
        .filter(db::cart_contents::aggregate::Column::CartId.eq(cart_id))
        .all(&tx);

    let (products, aggregate) = futures::future::try_join(products, aggregate).await?;

    for product in products {
        let provider = state.provider_from_id(product.provider_id);
        let search_product = state
            .providers
            .search_product_by_id(provider, &product.provider_product)
            .await?;

        let new_tally = product.quantity as u32 * search_product.display_price;
        result
            .entry(provider)
            .and_modify(|tally| *tally += new_tally)
            .or_insert(new_tally);
    }

    for (agg_ingredient, products) in aggregate {
        for product in products {
            let provider = state.provider_from_id(product.provider_id);
            let search_product = state
                .providers
                .search_product_by_id(provider, &product.provider_ingr_id)
                .await?;

            let new_tally = agg_ingredient.quantity as u32 * search_product.display_price;
            result
                .entry(provider)
                .and_modify(|tally| *tally += new_tally)
                .or_insert(new_tally);
        }
    }

    Ok(result)
}
