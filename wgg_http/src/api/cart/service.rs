use crate::api::{GraphqlResult, State};
use crate::db;
use crate::db::Id;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use std::collections::HashMap;
use std::ops::{Add, AddAssign};
use wgg_providers::models::{CentPrice, Provider};

/// Calculate the total tally of the given cart for all providers that are part of that cart.
#[tracing::instrument(skip(db, state))]
pub async fn calculate_tallies(
    db: &impl ConnectionTrait,
    cart_id: Id,
    state: &State,
) -> GraphqlResult<HashMap<Provider, TallyPriceInfo>> {
    let mut result = HashMap::with_capacity(state.db_providers.len());

    let products = db::cart_contents::raw_product::Entity::find()
        .filter(db::cart_contents::raw_product::Column::CartId.eq(cart_id))
        .all(db);

    let aggregate = db::cart_contents::aggregate::Entity::find()
        .find_with_related(db::agg_ingredients_links::Entity)
        .filter(db::cart_contents::aggregate::Column::CartId.eq(cart_id))
        .all(db);

    let (products, aggregate) = futures::future::try_join(products, aggregate).await?;

    for product in products {
        let provider = state.provider_from_id(product.provider_id);

        let search_product = state
            .providers
            .search_product(provider, &product.provider_product)
            .await?;

        let new_tally = TallyPriceInfo {
            full_price: search_product.full_price,
            display_price: product.quantity as u32 * search_product.display_price,
            discount: search_product.full_price - search_product.display_price,
        };

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
                .search_product(provider, &product.provider_ingr_id)
                .await?;

            let new_tally = TallyPriceInfo {
                full_price: search_product.full_price,
                display_price: agg_ingredient.quantity as u32 * search_product.display_price,
                discount: search_product.full_price - search_product.display_price,
            };

            result
                .entry(provider)
                .and_modify(|tally| *tally += new_tally)
                .or_insert(new_tally);
        }
    }

    Ok(result)
}

#[derive(Copy, Clone)]
pub struct TallyPriceInfo {
    pub full_price: CentPrice,
    pub display_price: CentPrice,
    pub discount: CentPrice,
}

impl Add for TallyPriceInfo {
    type Output = TallyPriceInfo;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            full_price: self.full_price + rhs.full_price,
            display_price: self.display_price + rhs.display_price,
            discount: self.discount + rhs.discount,
        }
    }
}

impl AddAssign for TallyPriceInfo {
    fn add_assign(&mut self, rhs: Self) {
        self.full_price += rhs.full_price;
        self.display_price += rhs.display_price;
        self.discount += rhs.discount;
    }
}
