use crate::models::{
    AllergyTags, AllergyType, CentPrice, FreshLabel, IngredientInfo, ItemInfo, ItemType, MoreButton, NutritionalInfo,
    NutritionalItem, PrepTime, PromotionProduct, SaleLabel, SaleValidity, SubNutritionalItem, UnavailableItem,
    UnitPrice, WggProduct, WggSaleCategory,
};
use crate::providers::common_bridge::parse_quantity;
use crate::providers::{common_bridge, ProviderInfo};
use crate::{OffsetPagination, Provider, WggAutocomplete, WggSearchProduct};
use chrono::{Datelike, LocalResult, NaiveDate, TimeZone};
use std::borrow::Cow;
use wgg_picnic::models::{Decorator, ImageSize, SubCategory, UnavailableReason};
use wgg_picnic::PicnicApi;

use crate::Result;

/// A separate bridge struct to allow for easier caching.
pub(crate) struct PicnicBridge {
    pub api: PicnicApi,
}

impl PicnicBridge {
    pub fn new(api: PicnicApi) -> Self {
        PicnicBridge { api }
    }
}

#[async_trait::async_trait]
impl ProviderInfo for PicnicBridge {
    fn provider() -> Provider {
        Provider::Picnic
    }

    fn logo_url(&self) -> Cow<'static, str> {
        "https://upload.wikimedia.org/wikipedia/commons/0/01/Picnic_logo.svg".into()
    }

    #[tracing::instrument(name = "picnic_autocomplete", level = "debug", skip(self))]
    async fn autocomplete(&self, query: &str) -> Result<Vec<WggAutocomplete>> {
        let result = self.api.suggestions(query).await?;

        #[cfg(feature = "trace-original-api")]
        tracing::trace!("Picnic Autocomplete: {:#?}", result);

        Ok(result
            .into_iter()
            .map(|i| WggAutocomplete { name: i.suggestion })
            .collect())
    }

    #[tracing::instrument(name = "picnic_search", level = "debug", skip(self, _offset))]
    async fn search(&self, query: &str, _offset: Option<u32>) -> Result<OffsetPagination<WggSearchProduct>> {
        let result = self.api.search(query).await?;

        #[cfg(feature = "trace-original-api")]
        tracing::trace!("Picnic Search: {:#?}", result);

        let result: Vec<WggSearchProduct> = result
            .into_iter()
            .flat_map(|res| {
                res.items.into_iter().filter_map(|item| {
                    if let wgg_picnic::models::SearchItem::SingleArticle(article) = item {
                        Some(parse_picnic_item_to_search_item(&self.api, article))
                    } else {
                        None
                    }
                })
            })
            .collect();

        let offset = OffsetPagination {
            total_items: result.len(),
            items: result,
            offset: 0,
        };

        Ok(offset)
    }

    async fn product(&self, product_id: &str) -> Result<WggProduct> {
        let result = self.api.product(product_id).await?;

        #[cfg(feature = "trace-original-api")]
        tracing::trace!("Picnic Product: {:#?}", result);

        parse_picnic_full_product_to_product(&self.api, result.product_details)
    }

    async fn promotions(&self) -> Result<Vec<WggSaleCategory>> {
        let result = self.api.promotions(None, 3).await?;

        #[cfg(feature = "trace-original-api")]
        tracing::trace!("Picnic Promotions: {:#?}", result);

        parse_picnic_promotions(&self.api, result)
    }

    async fn promotions_sublist(&self, sublist_id: &str) -> Result<OffsetPagination<WggSearchProduct>> {
        let result = self.api.promotions(Some(sublist_id), 3).await?;

        #[cfg(feature = "trace-original-api")]
        tracing::trace!("Picnic Promotions Sublist: {:#?}", result);
        // When querying for a sublist with depth>0 we just get a raw array of SingleArticles
        let result: Vec<WggSearchProduct> = result
            .into_iter()
            .flat_map(|res| {
                if let SubCategory::SingleArticle(article) = res {
                    Some(parse_picnic_item_to_search_item(&self.api, article))
                } else {
                    None
                }
            })
            .collect();

        let offset = OffsetPagination {
            total_items: result.len(),
            items: result,
            offset: 0,
        };

        Ok(offset)
    }
}

fn parse_picnic_promotions(
    picnic_api: &wgg_picnic::PicnicApi,
    promotions: Vec<SubCategory>,
) -> Result<Vec<WggSaleCategory>> {
    Ok(promotions
        .into_iter()
        .flat_map(|item| match item {
            SubCategory::Category(cat) => Some(cat),
            _ => {
                tracing::warn!(?item, "Expected categories in promotion parsing, but found articles");
                None
            }
        })
        .map(|category| {
            let mut result = WggSaleCategory {
                id: category.id,
                name: category.name,
                image_urls: vec![],
                limited_items: vec![],
                decorators: vec![],
                provider: Provider::Picnic,
            };

            // Decorators
            for dec in category.decorators {
                if let Decorator::MoreButton { images, .. } = &dec {
                    result.image_urls = images
                        .iter()
                        .map(|id| picnic_api.image_url(id, ImageSize::Medium))
                        .collect();
                }

                parse_decorator(picnic_api, dec, &mut result.decorators, None, None);
            }

            result.limited_items = category
                .items
                .into_iter()
                .flat_map(|item| match item {
                    SubCategory::SingleArticle(itm) => Some(itm),
                    _ => {
                        tracing::warn!(
                            ?item,
                            "Expected single article in promotion parsing, but found articles"
                        );
                        None
                    }
                })
                .map(|item| parse_picnic_item_to_search_item(picnic_api, item))
                .map(PromotionProduct::Product)
                .collect();

            result
        })
        .collect())
}

/// Parse a full picnic [wgg_picnic::models::ProductDetails] to our normalised [Product]
fn parse_picnic_full_product_to_product(
    picnic_api: &PicnicApi,
    product: wgg_picnic::models::ProductDetails,
) -> Result<WggProduct> {
    // Note that Picnic's 'display_price' is equivalent to our 'full_price'.
    let mut result = WggProduct {
        id: product.id,
        name: product.name,
        description: product.description,
        full_price: product.original_price.unwrap_or(product.display_price),
        display_price: product.display_price,
        // Will be parsed
        unit_quantity: Default::default(),
        // Will be parsed
        unit_price: None,
        // Will be parsed
        available: true,
        image_urls: product
            .image_ids
            .into_iter()
            .map(|url| picnic_api.image_url(url, ImageSize::Medium))
            .collect(),
        // Will be parsed
        ingredients: Vec::new(),
        // Will be parsed
        nutritional: None,
        // Will be parsed
        allergy_info: Vec::new(),
        // Will be parsed
        decorators: Vec::new(),
        // Will be parsed
        additional_items: Vec::new(),
        provider: Provider::Picnic,
    };

    // Parse unit quantity
    if let Some(quantity) = parse_quantity(&product.unit_quantity) {
        result.unit_quantity = quantity;
    } else {
        // Since we couldn't parse a 'normal' quantity it might be of an unconventional form such as:
        // `4-6 pers | 30 mins`, we can extract the prep time!
        if let Some(minutes) = parse_prep_time(&product.unit_quantity) {
            result.decorators.push(crate::models::WggDecorator::PrepTime(PrepTime {
                time_minutes: minutes,
            }));
        }
    }

    // Parse unit price quantity
    if let Some(unit_price_str) = &product.unit_quantity_sub {
        result.unit_price = parse_unit_price(unit_price_str)
            .or_else(|| common_bridge::derive_unit_price(&result.unit_quantity, result.display_price));
    } else {
        result.unit_price = common_bridge::derive_unit_price(&result.unit_quantity, result.display_price)
    }

    // Parse Ingredients
    if let Some(blob) = product.ingredients_blob {
        result.ingredients = parse_picnic_ingredient_blob(&blob);
    }

    // Parse nutritional, only parse if there is something of note.
    if product.nutritional_info_unit.is_some() || !product.nutritional_values.is_empty() {
        result.nutritional = NutritionalInfo {
            info_unit: product.nutritional_info_unit.unwrap_or_else(|| "per 100g".to_string()),
            items: product
                .nutritional_values
                .into_iter()
                .map(|item| NutritionalItem {
                    name: item.name,
                    value: item.value,
                    sub_values: item
                        .sub_values
                        .into_iter()
                        .map(|item| SubNutritionalItem {
                            name: item.name,
                            value: item.value,
                        })
                        .collect(),
                })
                .collect(),
        }
        .into();
    }

    // Parse allergy info
    result.allergy_info = product
        .tags
        .into_iter()
        .map(|item| AllergyTags {
            name: item.name,
            contains: {
                // Examples: "Dit product bevat soja." vs. "Dit product kan selderij bevatten."
                if item.description.contains("kan") {
                    AllergyType::MayContain
                } else {
                    AllergyType::Contains
                }
            },
        })
        .collect();

    // Parse fresh label
    if let Some(fresh) = product.fresh_label {
        let multiplier = match &*fresh.unit {
            "DAYS" | "days" | "day" => 1,
            "WEEKS" | "weeks" | "week" => 7,
            "MONTHS" | "months" | "month" => 30,
            "YEARS" | "years" | "year" => 365,
            _ => 1,
        };

        let fresh_label = FreshLabel {
            days_fresh: fresh.number * multiplier,
        };

        result
            .decorators
            .push(crate::models::WggDecorator::FreshLabel(fresh_label))
    }

    // Parse remaining decorators
    for dec in product.decorators {
        parse_decorator(
            picnic_api,
            dec,
            &mut result.decorators,
            Some(&mut result.display_price),
            Some(&mut result.available),
        )
    }

    // Parse items
    result.additional_items = product
        .items
        .into_iter()
        .flat_map(|mut item| {
            // We already parse ingredients earlier.
            if item.id == "ingredients" {
                return None;
            }
            // Picnic only ever seems to return one item in the sub-items list, so we'll just assume that's ok!
            let item_of_interest = item.items.pop()?;

            let item_type = match &*item_of_interest.id {
                "preparation_advice" => ItemType::PreparationAdvice,
                "countries_of_origin" => ItemType::CountryOfOrigin,
                _ => ItemType::AdditionalInfo,
            };

            item_of_interest.text.map(|text| ItemInfo { item_type, text })
        })
        .collect();

    // Filter out duplicate country_of_origin
    if !product.additional_info.is_empty() && !product.additional_info.contains("herkomst") {
        result.additional_items.push(ItemInfo {
            item_type: ItemType::AdditionalInfo,
            text: product.additional_info,
        })
    }

    Ok(result)
}

/// Parse a full picnic [wgg_picnic::models::SingleArticle] to our normalised [SearchItem]
fn parse_picnic_item_to_search_item(
    picnic_api: &PicnicApi,
    article: wgg_picnic::models::SingleArticle,
) -> WggSearchProduct {
    // Note that Picnic's 'display_price' is equivalent to our 'full_price'.
    let mut result = WggSearchProduct {
        id: article.id,
        name: article.name,
        full_price: article.display_price,
        display_price: article.display_price,
        unit_quantity: Default::default(),
        unit_price: None,
        available: true,
        image_url: Some(picnic_api.image_url(article.image_id, wgg_picnic::models::ImageSize::Medium)),
        decorators: Vec::new(),
        provider: Provider::Picnic,
    };

    // Parse remaining decorators
    for dec in article.decorators {
        parse_decorator(
            picnic_api,
            dec,
            &mut result.decorators,
            Some(&mut result.display_price),
            Some(&mut result.available),
        )
    }

    // Parse unit quantity
    if let Some(quantity) = parse_quantity(&article.unit_quantity) {
        result.unit_quantity = quantity;
    } else {
        // Since we couldn't parse a 'normal' quantity it might be of an unconventional form such as:
        // `4-6 pers | 30 mins`, we can extract the prep time!
        if let Some(minutes) = parse_prep_time(&article.unit_quantity) {
            result.decorators.push(crate::models::WggDecorator::PrepTime(PrepTime {
                time_minutes: minutes,
            }));
        }
    }

    // Parse unit price quantity
    if let Some(unit_price_str) = &article.unit_quantity_sub {
        result.unit_price = parse_unit_price(unit_price_str)
            .or_else(|| common_bridge::derive_unit_price(&result.unit_quantity, result.display_price));
    } else {
        result.unit_price = common_bridge::derive_unit_price(&result.unit_quantity, result.display_price)
    }

    result
}

// Encourage inlining to get rid of the Option costs.
#[inline(always)]
pub fn parse_decorator(
    picnic_api: &PicnicApi,
    decorator: Decorator,
    result: &mut Vec<crate::models::WggDecorator>,
    set_display_price: Option<&mut u32>,
    set_available: Option<&mut bool>,
) {
    match decorator {
        // If we already parsed it above, we don't want to do it again!
        Decorator::FreshLabel { period }
            if !result
                .iter()
                .any(|i| matches!(i, crate::models::WggDecorator::FreshLabel(_))) =>
        {
            if let Some(days_fresh) = parse_days_fresh(&period) {
                let fresh_label = FreshLabel { days_fresh };

                result.push(crate::models::WggDecorator::FreshLabel(fresh_label))
            }
        }
        Decorator::Label { text } => {
            let sale_label = SaleLabel { text };

            result.push(crate::models::WggDecorator::SaleLabel(sale_label))
        }
        Decorator::Price { display_price } => {
            // Decorator price is the price *including* current sales if available.
            if let Some(dp) = set_display_price {
                *dp = display_price
            }
        }
        Decorator::ValidityLabel { valid_until } => {
            if let LocalResult::Single(valid_until) = chrono::Utc.from_local_datetime(&valid_until.and_hms(23, 59, 59))
            {
                let valid_from =
                    NaiveDate::from_isoywd(valid_until.year(), valid_until.iso_week().week(), chrono::Weekday::Mon)
                        .and_hms(0, 0, 0);
                let valid_from = if let Some(time) = chrono::Utc.from_local_datetime(&valid_from).single() {
                    time
                } else {
                    return;
                };
                let sale_validity = SaleValidity {
                    valid_from,
                    valid_until,
                };

                result.push(crate::models::WggDecorator::SaleValidity(sale_validity))
            }
        }
        Decorator::Unavailable {
            reason,
            replacements,
            explanation,
        } => {
            let unavailable = UnavailableItem {
                reason: match reason {
                    UnavailableReason::OutOfAssortment => crate::models::UnavailableReason::OutOfAssortment,
                    UnavailableReason::OutOfSeason => crate::models::UnavailableReason::OutOfSeason,
                    UnavailableReason::TemporarilyUnavailable => {
                        crate::models::UnavailableReason::TemporarilyUnavailable
                    }
                    _ => crate::models::UnavailableReason::Unknown,
                },
                explanation_short: explanation.short_explanation.into(),
                explanation_long: explanation.long_explanation.into(),
                replacements: replacements
                    .into_iter()
                    .map(|item| parse_picnic_item_to_search_item(picnic_api, item))
                    .collect(),
            };

            if let Some(available) = set_available {
                *available = false;
            }
            result.push(crate::models::WggDecorator::Unavailable(unavailable))
        }
        Decorator::MoreButton { images, .. } => {
            let more_button = MoreButton {
                images: images
                    .into_iter()
                    .map(|id| picnic_api.image_url(id, ImageSize::Medium))
                    .collect(),
            };

            result.push(crate::models::WggDecorator::MoreButton(more_button))
        }
        _ => {}
    }
}

/// Try to parse ingredient blob in the form: `71% tomaat, ui, wortel, 6,6% tomatenpuree (tomatenpuree, zout), etc`
fn parse_picnic_ingredient_blob(blob: &str) -> Vec<IngredientInfo> {
    // Picnic's ingredient blob is uncharacteristically unstructured, so we have to break it apart ourselves.
    // It has the form "71% tomaat, ui, wortel, 6,6% tomatenpuree (tomatenpuree, zout), etc"

    //TODO: Fix stuff between brackets. This unfortunately needs a context-free parser with look-ahead.

    let result = blob
        .split(", ")
        .filter(|s| !s.is_empty())
        .map(|ingr| IngredientInfo {
            name: ingr.trim().trim_end_matches('.').to_string(),
        })
        .collect();

    result
}

/// Try to parse the unit string as a prep time in minutes
fn parse_prep_time(unit_str: &str) -> Option<u32> {
    // Split from `4-6 pers | 30 mins`
    let (_, minutes) = unit_str.split_once('|')?;
    // Remainder: ` 30 mins` -> `30` -> u32
    minutes.split_whitespace().next()?.parse().ok()
}

/// Try to parse the provided unit price in the format `€16.54/l` or `€13.54/kg`.
///
/// Invalid input will return [None]
fn parse_unit_price(unit_price: &str) -> Option<UnitPrice> {
    if let Some((price_component, unit_component)) = unit_price.split_once('/') {
        let cent_price = parse_euro_price(price_component);
        let unit = common_bridge::parse_unit_component(unit_component);

        cent_price.zip(unit).map(|(price, unit)| UnitPrice { unit, price })
    } else {
        None
    }
}

/// Try to parse the provided price in the format `€16.54` or `2.32` as a price in cents.
///
/// Invalid input will return [None]
fn parse_euro_price(price: &str) -> Option<CentPrice> {
    let trimmed = price.trim_start_matches('€');
    let float_price: f64 = trimmed.parse().ok()?;
    let cent_price = (float_price * 100.).round() as u32;

    Some(cent_price)
}

/// Parse the days fresh in the format `P5D` or `P7D` (5 days and 7 days respectively).
fn parse_days_fresh(period: &str) -> Option<u32> {
    let price: String = period.chars().filter(|&ch| ch != 'P' && ch != 'D').collect();

    price.parse().ok()
}

#[cfg(test)]
mod test {
    use crate::models::{Unit, UnitPrice};
    use crate::providers::picnic_bridge::{
        parse_days_fresh, parse_euro_price, parse_picnic_ingredient_blob, parse_prep_time, parse_unit_price,
    };
    use std::{println, vec};

    #[test]
    pub fn test_parse_price() {
        let prices = vec!["€16.57", "€19.22", "€19", "2.32"];

        assert_eq!(
            prices.into_iter().flat_map(parse_euro_price).collect::<Vec<_>>(),
            vec![1657, 1922, 1900, 232]
        );
    }

    #[test]
    pub fn test_parse_unit_price() {
        let unit_prices = vec!["€13.08/kg", "€1.64/l", "€0.20/stuk"];
        let expected = vec![
            UnitPrice {
                unit: Unit::KiloGram,
                price: 1308,
            },
            UnitPrice {
                unit: Unit::Liter,
                price: 164,
            },
            UnitPrice {
                unit: Unit::Piece,
                price: 20,
            },
        ];

        assert_eq!(
            unit_prices.into_iter().flat_map(parse_unit_price).collect::<Vec<_>>(),
            expected
        );
    }

    #[test]
    fn test_parse_days_fresh() {
        let periods = vec!["P5D", "P7D"];

        assert_eq!(
            periods.into_iter().flat_map(parse_days_fresh).collect::<Vec<_>>(),
            vec![5, 7]
        );
    }

    #[test]
    fn test_parse_prep_time() {
        let periods = vec!["4-6 pers | 30 min", "4 pers |  25 min", "3-4 pers | 40 min"];

        assert_eq!(
            periods.into_iter().flat_map(parse_prep_time).collect::<Vec<_>>(),
            vec![30, 25, 40]
        );
    }

    #[test]
    fn test_parse_ingredients() {
        let ingredient_str = "71% tomaat, ui, wortel, 6,6% tomatenpuree (tomatenpuree, zout), etc, ";

        let output = parse_picnic_ingredient_blob(ingredient_str);

        println!("Output: {:#?}", output);
    }
}
