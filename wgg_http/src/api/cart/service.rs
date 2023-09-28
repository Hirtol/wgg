use crate::api::error::GraphqlError;
use crate::api::{AppState, GraphqlResult};
use crate::db;
use crate::db::Id;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use std::collections::HashMap;
use wgg_providers::models::sale_types::SaleType;
use wgg_providers::models::{
    CentPrice, PriceInfo, Provider, SaleInformation, SaleResolutionStrategy, SublistId, WggSearchProduct,
};

/// Get the direct quantity (ignoring Aggregate products) of the given product in the given cart.
///
/// For bulk queries please refer to [get_products_quantity].
pub async fn get_direct_product_quantity(
    db: &impl ConnectionTrait,
    cart_id: Option<Id>,
    user_id: Id,
    provider_id: Id,
    product_id: &str,
) -> GraphqlResult<Option<u32>> {
    let cart_content = db::cart_contents::raw_product::Entity::find()
        .filter(db::cart_contents::raw_product::Column::ProviderId.eq(provider_id))
        .filter(db::cart_contents::raw_product::Column::ProviderProduct.eq(product_id))
        .left_join(db::cart::Entity)
        .filter(db::cart::is_cart_or_active_cart(cart_id, user_id))
        .one(db)
        .await?;

    Ok(cart_content.map(|i| i.quantity as u32))
}

pub async fn get_aggregate_product_quantity(
    db: &impl ConnectionTrait,
    cart_id: Option<Id>,
    user_id: Id,
    aggregate_id: Id,
) -> GraphqlResult<Option<u32>> {
    let cart_content = db::cart_contents::aggregate::Entity::find()
        .left_join(db::agg_ingredients_links::Entity)
        .left_join(db::cart::Entity)
        .filter(db::cart::is_cart_or_active_cart(cart_id, user_id))
        .filter(db::agg_ingredients_links::related_aggregate(aggregate_id))
        .one(db)
        .await?;

    Ok(cart_content.map(|i| i.quantity as u32))
}

/// Get the direct quantity (ignoring Aggregate products) of the given product(s) in the given cart.
#[allow(dead_code)]
pub async fn get_products_quantity(
    db: &impl ConnectionTrait,
    cart_id: Option<Id>,
    user_id: Id,
    product_ids: impl IntoIterator<Item = &str>,
) -> GraphqlResult<HashMap<String, u32>> {
    let cart_content = db::cart_contents::raw_product::Entity::find()
        .filter(db::cart_contents::raw_product::Column::ProviderProduct.is_in(product_ids))
        .left_join(db::cart::Entity)
        .filter(db::cart::is_cart_or_active_cart(cart_id, user_id))
        .all(db)
        .await?;

    Ok(cart_content
        .into_iter()
        .map(|item| (item.provider_product, item.quantity as u32))
        .collect())
}

/// Calculate the total tally of the given cart for all providers that are part of that cart.
#[tracing::instrument(skip(db, state))]
pub async fn calculate_tallies(
    db: &impl ConnectionTrait,
    cart_id: Id,
    state: &AppState,
) -> GraphqlResult<HashMap<Provider, TallyPriceInfo>> {
    let mut result: HashMap<Provider, TallyPriceInfo> = HashMap::with_capacity(state.db_providers.len());
    let mut sale_items: HashMap<SublistId, SaleTracking> = HashMap::new();

    let products = db::cart_contents::raw_product::Entity::find()
        .filter(db::cart_contents::raw_product::Column::CartId.eq(cart_id))
        .all(db);

    let aggregate = db::cart_contents::aggregate::Entity::find()
        .find_with_related(db::agg_ingredients_links::Entity)
        .filter(db::cart_contents::aggregate::Column::CartId.eq(cart_id))
        .all(db);

    let (products, aggregate) = futures::future::try_join(products, aggregate).await?;

    let mut add_sale_item = |search_product: WggSearchProduct, quantity: u32| {
        let Some(sale) = &search_product.sale_information else {
            return Ok(());
        };

        let provider = search_product.provider;
        let Some(sale_id) = state.providers.product_sale_id(provider, &search_product.id) else {
            return Err(GraphqlError::InternalError(format!(
                "Product: {:?} - {} - has a sale with no associated sale!",
                provider, search_product.id
            )));
        };

        sale_items
            .entry(sale_id)
            .and_modify(|tracking| {
                tracking.items.push(ProductWithQuantity {
                    quantity,
                    item: search_product.clone(),
                })
            })
            .or_insert_with(|| SaleTracking {
                items: vec![ProductWithQuantity {
                    quantity,
                    item: search_product.clone(),
                }],
                sale_info: sale.clone(),
                provider,
            });
        Ok(())
    };

    for product in products {
        let provider = state.provider_from_id(product.provider_id);

        let search_product = state
            .providers
            .search_product(provider, &product.provider_product)
            .await?;

        let original_price = product.quantity as u32 * search_product.price_info.original_price;

        // Handle sale look-up.
        if let Err(e) = add_sale_item(search_product, product.quantity as u32) {
            tracing::warn!(?e, "Failed to handle sale item")
        }

        result
            .entry(provider)
            .and_modify(|tally| tally.original_price += original_price)
            .or_insert(TallyPriceInfo {
                original_price,
                discount: 0,
            });
    }

    for (agg_ingredient, products) in aggregate {
        for product in products {
            let provider = state.provider_from_id(product.provider_id);
            let search_product = state
                .providers
                .search_product(provider, &product.provider_ingr_id)
                .await?;

            let original_price = agg_ingredient.quantity as u32 * search_product.price_info.original_price;

            // Handle sale look-up.
            if let Err(e) = add_sale_item(search_product, agg_ingredient.quantity as u32) {
                tracing::warn!(?e, "Failed to handle aggregate item")
            }

            result
                .entry(provider)
                .and_modify(|tally| tally.original_price += original_price)
                .or_insert(TallyPriceInfo {
                    original_price,
                    discount: 0,
                });
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

    for (_, sale) in sale_items {
        let Some(sale_type) = sale.sale_info.sale_type else {
            // Can't do anything, assume the pessimistic original price
            continue;
        };

        match sale_type {
            SaleType::NumPlusNumFree(data) => {
                let required = data.required.get() as u32;
                let total_required = required + data.free.get() as u32;

                handle_group(
                    tally_map,
                    &mut item_group_results,
                    sale.provider,
                    total_required,
                    sale.items,
                    |total_original_price| (total_original_price / total_required) * required,
                );
            }
            SaleType::NumthPercentOff(data) => {
                let required = data.required.get() as u32;
                let total_required = required;

                handle_group(
                    tally_map,
                    &mut item_group_results,
                    sale.provider,
                    total_required,
                    sale.items,
                    |total_original_price| {
                        total_original_price
                            - ((total_original_price * data.last_percent_off.get() as u32) / (100 * required))
                    },
                );
            }
            SaleType::NumForPrice(data) => {
                let total_required = data.required.get() as u32;

                handle_group(
                    tally_map,
                    &mut item_group_results,
                    sale.provider,
                    total_required,
                    sale.items,
                    |_| data.price,
                );
            }
            SaleType::NumPercentOff(_) | SaleType::NumEuroPrice(_) | SaleType::NumEuroOff(_) => {
                // Since this is a single item we can assume the `display_price` property is well preserved.
                for product in sale.items {
                    let original_price = product.item.price_info.original_price * product.quantity;
                    let display_price = product.item.price_info.display_price * product.quantity;

                    tally_map
                        .entry(product.item.provider)
                        .and_modify(|tally| tally.discount += original_price - display_price);

                    let sale_group = SaleItemGroup {
                        price_info: PriceInfo {
                            display_price,
                            original_price,
                            unit_price: None,
                        },
                        items: vec![ProductWithQuantity {
                            quantity: product.quantity,
                            item: product.item,
                        }],
                    };

                    item_group_results.push(sale_group);
                }
            }
        }
    }

    Ok(item_group_results)
}

/// Handle a full [SaleTracking] and update the `tally_map`'s `discount`.
///
/// Additionally, the groupings of the items within the `items` [Vec] are pushed to `group_results`.
///
/// # Arguments
///
/// * `final_price_calc` - Takes the `total_original_price` of all items within a [SaleItemGroup] (which is a sub-group of the overal [SaleTracking])
/// and expects the final price of the current promotion taking into account said original price.
fn handle_group(
    tally_map: &mut HashMap<Provider, TallyPriceInfo>,
    group_results: &mut Vec<SaleItemGroup>,
    provider: Provider,
    total_required: u32,
    mut items: Vec<ProductWithQuantity>,
    mut final_price_calc: impl FnMut(CentPrice) -> CentPrice,
) {
    let mut total_qualifying_items: u32 = items.iter().map(|product| product.quantity).sum();
    let sort_items_on_price =
        |a: &WggSearchProduct, b: &WggSearchProduct| a.price_info.original_price.cmp(&b.price_info.original_price);

    if total_qualifying_items >= total_required {
        match provider.get_metadata().sale_strategy {
            SaleResolutionStrategy::Opportunistic => {
                items.sort_by(|a, b| sort_items_on_price(&b.item, &a.item));
            }
            SaleResolutionStrategy::Pessimistic => {
                items.sort_by(|a, b| sort_items_on_price(&a.item, &b.item));
            }
        }

        while total_qualifying_items >= total_required {
            total_qualifying_items -= total_required;
            let mut still_required = total_required;

            let item_group = items
                .iter_mut()
                .filter(|product| product.quantity > 0)
                .take_while(|product| {
                    if still_required == 0 {
                        false
                    } else {
                        still_required -= still_required.min(product.quantity);
                        true
                    }
                });

            let mut final_required = total_required;
            let mut total_original_price = 0;
            let mut items = Vec::with_capacity(total_required as usize);

            for product in item_group {
                let used_quantity = final_required.min(product.quantity);

                final_required -= used_quantity;
                product.quantity -= used_quantity;

                total_original_price += product.item.price_info.original_price * used_quantity;

                items.push(ProductWithQuantity {
                    quantity: used_quantity,
                    item: product.item.clone(),
                });
            }

            let final_price = final_price_calc(total_original_price);
            let sale_group = SaleItemGroup {
                price_info: PriceInfo {
                    display_price: final_price,
                    original_price: total_original_price,
                    unit_price: None,
                },
                items,
            };

            tally_map
                .entry(provider)
                .and_modify(|tally| tally.discount += total_original_price - final_price);
            group_results.push(sale_group)
        }
    }
}

struct SaleTracking {
    items: Vec<ProductWithQuantity>,
    sale_info: SaleInformation,
    provider: Provider,
}

#[derive(Debug)]
pub struct SaleItemGroup {
    price_info: PriceInfo,
    items: Vec<ProductWithQuantity>,
}

#[derive(Debug)]
pub struct ProductWithQuantity {
    quantity: u32,
    item: WggSearchProduct,
}

#[derive(Copy, Clone)]
pub struct TallyPriceInfo {
    pub original_price: CentPrice,
    pub discount: CentPrice,
}
