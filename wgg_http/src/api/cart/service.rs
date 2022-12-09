use crate::api::error::GraphqlError;
use crate::api::{GraphqlResult, State};
use crate::db;
use crate::db::Id;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use std::collections::HashMap;
use std::ops::{Add, AddAssign};
use wgg_providers::models::sale_types::SaleType;
use wgg_providers::models::{
    CentPrice, PriceInfo, Provider, SaleInformation, SaleResolutionStrategy, SublistId, WggSearchProduct,
};

/// Calculate the total tally of the given cart for all providers that are part of that cart.
#[tracing::instrument(skip(db, state))]
pub async fn calculate_tallies(
    db: &impl ConnectionTrait,
    cart_id: Id,
    state: &State,
) -> GraphqlResult<HashMap<Provider, TallyPriceInfo>> {
    let mut result = HashMap::with_capacity(state.db_providers.len());
    let mut sale_items: HashMap<SublistId, SaleTracking> = HashMap::new();

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
            original_price: product.quantity as u32 * search_product.price_info.original_price,
            discount: 0,
        };

        // Handle sale look-up.
        if let Some(sale) = &search_product.sale_information {
            let Some(sale_id) = state.providers.product_sale_id(search_product.provider, &search_product.id) else {
                return Err(GraphqlError::InternalError(format!("Product: {:?} - {} - has a sale with no associated sale!", search_product.provider, search_product.id)));
            };

            sale_items
                .entry(sale_id)
                .and_modify(|tracking| tracking.items.push((product.quantity as u32, search_product.clone())))
                .or_insert_with(|| SaleTracking {
                    items: vec![(product.quantity as u32, search_product.clone())],
                    sale_info: sale.clone(),
                    provider,
                });
        }

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
                original_price: agg_ingredient.quantity as u32 * search_product.price_info.original_price,
                discount: 0,
            };

            // Handle sale look-up.
            if let Some(sale) = &search_product.sale_information {
                let Some(sale_id) = state.providers.product_sale_id(search_product.provider, &search_product.id) else {
                    return Err(GraphqlError::InternalError(format!("Product: {:?} - {} - has a sale with no associated sale!", search_product.provider, search_product.id)));
                };

                sale_items
                    .entry(sale_id)
                    .and_modify(|tracking| {
                        tracking
                            .items
                            .push((agg_ingredient.quantity as u32, search_product.clone()))
                    })
                    .or_insert_with(|| SaleTracking {
                        items: vec![(agg_ingredient.quantity as u32, search_product.clone())],
                        sale_info: sale.clone(),
                        provider,
                    });
            }

            result
                .entry(provider)
                .and_modify(|tally| *tally += new_tally)
                .or_insert(new_tally);
        }
    }

    // TODO: Expose the sale item groups for the front-end to beautify!
    let _ = handle_sale_logic(&mut result, sale_items)?;

    Ok(result)
}

fn handle_sale_logic(
    tally_map: &mut HashMap<Provider, TallyPriceInfo>,
    sale_items: HashMap<SublistId, SaleTracking>,
) -> GraphqlResult<Vec<SaleItemGroup>> {
    let mut item_group_results = Vec::new();

    let sort_items_on_price =
        |a: &WggSearchProduct, b: &WggSearchProduct| a.price_info.original_price.cmp(&b.price_info.original_price);

    for (_, mut sale) in sale_items {
        if let Some(sale_type) = sale.sale_info.sale_type {
            match sale_type {
                SaleType::NumPlusNumFree(data) => {
                    let required = data.required.get() as u32;
                    let total_required = required + data.free.get() as u32;

                    let mut total_qualifying_items: u32 = sale.items.iter().map(|(quantity, _)| quantity).sum();

                    if total_qualifying_items >= total_required {
                        match sale.provider.get_metadata().sale_strategy {
                            SaleResolutionStrategy::Opportunistic => {
                                sale.items.sort_by(|a, b| sort_items_on_price(&b.1, &a.1));
                            }
                            SaleResolutionStrategy::Pessimistic => {
                                sale.items.sort_by(|a, b| sort_items_on_price(&a.1, &b.1));
                            }
                        }

                        while total_qualifying_items >= total_required {
                            total_qualifying_items -= total_required;
                            let mut still_required = total_required;

                            let item_group = sale.items.iter_mut().filter(|(quantity, _)| *quantity > 0).take_while(
                                |(quantity, _)| {
                                    if still_required == 0 {
                                        false
                                    } else {
                                        still_required -= still_required.min(*quantity);
                                        true
                                    }
                                },
                            );

                            let mut final_required = total_required;
                            let mut total_original_price = 0;
                            let mut items = Vec::with_capacity(total_required as usize);

                            for (quantity, item) in item_group {
                                let used_quantity = final_required.min(*quantity);

                                final_required -= used_quantity;
                                *quantity -= used_quantity;

                                total_original_price += item.price_info.original_price * used_quantity;

                                items.push(ProductWithQuantity {
                                    quantity: used_quantity,
                                    item: item.clone(),
                                });
                            }

                            let final_price = (total_original_price / total_required) * required;
                            let sale_group = SaleItemGroup {
                                price_info: PriceInfo {
                                    display_price: total_original_price,
                                    original_price: final_price,
                                    unit_price: None,
                                },
                                items,
                            };

                            tally_map
                                .entry(sale.provider)
                                .and_modify(|tally| tally.discount += total_original_price - final_price);
                            item_group_results.push(sale_group)
                        }
                    }
                }
                SaleType::NumPercentOff(_) => {}
                SaleType::NumthPercentOff(_) => {}
                SaleType::NumForPrice(_) => {}
                SaleType::NumEuroOff(_) => {}
                SaleType::NumEuroPrice(_) => {}
            }
        } else {
            // Can't properly handle this one, assume pessimistic case of original price.
        }
    }

    Ok(item_group_results)
}

#[derive(Debug)]
struct SaleItemGroup {
    price_info: PriceInfo,
    items: Vec<ProductWithQuantity>,
}

#[derive(Debug)]
struct ProductWithQuantity {
    quantity: u32,
    item: WggSearchProduct,
}

struct SaleTracking {
    items: Vec<(u32, WggSearchProduct)>,
    sale_info: SaleInformation,
    provider: Provider,
}

#[derive(Copy, Clone)]
pub struct TallyPriceInfo {
    pub original_price: CentPrice,
    pub discount: CentPrice,
}

impl Add for TallyPriceInfo {
    type Output = TallyPriceInfo;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            original_price: self.original_price + rhs.original_price,
            discount: self.discount + rhs.discount,
        }
    }
}

impl AddAssign for TallyPriceInfo {
    fn add_assign(&mut self, rhs: Self) {
        self.original_price += rhs.original_price;
        self.discount += rhs.discount;
    }
}
