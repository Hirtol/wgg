use crate::models::{
    AllergyTags, AllergyType, CentPrice, FreshLabel, IngredientInfo, ItemInfo, ItemType, MoreButton, NutritionalInfo,
    NutritionalItem, PrepTime, PriceInfo, PromotionProduct, SaleLabel, SaleValidity, SubNutritionalItem, TextType,
    UnavailableItem, UnitPrice, WggDecorator, WggProduct, WggSaleCategory,
};
use crate::providers::common_bridge::parse_quantity;
use crate::providers::{common_bridge, ProviderInfo, StaticProviderInfo};
use crate::{OffsetPagination, Provider, WggAutocomplete, WggSearchProduct};
use chrono::{Datelike, LocalResult, NaiveDate, TimeZone};
use itertools::Itertools;
use regex::Regex;
use std::borrow::Cow;
use wgg_picnic::models::{Body, Decorator, ImageSize, PmlComponent, SubCategory, UnavailableReason};
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

impl StaticProviderInfo for PicnicBridge {
    fn provider() -> Provider {
        Provider::Picnic
    }

    fn logo_url() -> Cow<'static, str> {
        "https://upload.wikimedia.org/wikipedia/commons/0/01/Picnic_logo.svg".into()
    }
}

#[async_trait::async_trait]
impl ProviderInfo for PicnicBridge {
    fn provider(&self) -> Provider {
        <Self as StaticProviderInfo>::provider()
    }

    fn logo_url(&self) -> Cow<'static, str> {
        <Self as StaticProviderInfo>::logo_url()
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

        parse_picnic_full_product_to_product(&self.api, result)
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
    product: wgg_picnic::models::ProductArticle,
) -> Result<WggProduct> {
    let mut result = WggProduct {
        id: product.id,
        name: product.name,
        description: {
            // Our model assumes there is always a description, so we'll just make an empty one if it doesn't exist.
            let desc = product.description.unwrap_or_default();
            let out = desc.main;

            if let Some(extra) = desc.extension {
                out + &extra
            } else {
                out
            }
        },
        price_info: PriceInfo {
            display_price: product.price_info.price,
            original_price: product.price_info.original_price.unwrap_or(product.price_info.price),
            // Will be parsed
            unit_price: None,
        },
        // Will be parsed
        unit_quantity: Default::default(),
        // Will be parsed
        available: true,
        image_urls: product
            .images
            .into_iter()
            .map(|url| picnic_api.image_url(url.image_id, ImageSize::Medium))
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
    if let Some(unit_price_str) = &product.price_info.base_price_text {
        result.price_info.unit_price = parse_unit_price(unit_price_str)
            .or_else(|| common_bridge::derive_unit_price(&result.unit_quantity, result.price_info.display_price));
    } else {
        result.price_info.unit_price =
            common_bridge::derive_unit_price(&result.unit_quantity, result.price_info.display_price)
    }

    // Parse ingredients
    if let Some(blob) = product.misc.iter().find(|i| i.header.text.contains("Ingrediënten")) {
        if let wgg_picnic::models::Body::Pml { pml_content } = &blob.body {
            if let wgg_picnic::models::PmlComponent::RichText(item) = &pml_content.component {
                result.ingredients = parse_picnic_ingredient_blob(&item.markdown).unwrap_or_default();
            } else {
                tracing::warn!(product=?result, "Failed to find a rich-text component for the ingredient blob")
            }
        } else {
            tracing::warn!(product=?result, "Failed to find a PML body for the ingredient blob")
        }
    }

    // Parse nutritional
    if let Some(blob) = product.misc.iter().find(|i| i.header.text.contains("Voedingswaarde")) {
        if let Body::NutritionalTable { nutritional_table } = &blob.body {
            result.nutritional = NutritionalInfo {
                info_unit: nutritional_table.default_unit.clone(),
                items: nutritional_table
                    .values
                    .iter()
                    .map(|item| NutritionalItem {
                        name: item.name.clone(),
                        value: item.value.clone(),
                        sub_values: item
                            .sub_values
                            .iter()
                            .map(|item| SubNutritionalItem {
                                name: item.name.clone(),
                                value: item.value.clone(),
                            })
                            .collect(),
                    })
                    .collect(),
            }
            .into();
        } else {
            tracing::warn!(product=?result, "Failed to find a NutritionTable body for the nutritional blob")
        }
    }

    // Parse allergy info
    result.allergy_info = product
        .allergies
        .allergy_contains
        .into_iter()
        .map(|item| AllergyTags {
            name: item.name,
            contains: AllergyType::Contains,
        })
        .chain(
            product
                .allergies
                .allergy_may_contain
                .into_iter()
                .map(|item| AllergyTags {
                    name: item,
                    contains: AllergyType::MayContain,
                }),
        )
        .collect();

    // Parse fresh label
    if let Some(fresh) = product.highlights.iter().find(|item| item.text.contains("dagen vers")) {
        static REGEX: once_cell::sync::Lazy<Regex> =
            once_cell::sync::Lazy::new(|| Regex::new(r#"(\d+) (dagen|dag|week|weken)"#).unwrap());

        for capture in REGEX.captures_iter(&fresh.text) {
            let (number, unit) = (capture.get(1).unwrap(), capture.get(2).unwrap());

            let multiplier = match unit.as_str() {
                "dagen" | "dag" => 1,
                "week" | "weken" => 7,
                _ => 1,
            };

            let fresh_label = FreshLabel {
                days_fresh: number.as_str().parse::<u32>().unwrap() * multiplier,
            };

            result
                .decorators
                .push(crate::models::WggDecorator::FreshLabel(fresh_label))
        }
    }

    // Parse remaining decorators
    for dec in product.decorators {
        parse_decorator(
            picnic_api,
            dec,
            &mut result.decorators,
            Some(&mut result.price_info.display_price),
            Some(&mut result.available),
        )
    }

    if let Some(promo) = product.labels.promo {
        result
            .decorators
            .push(WggDecorator::SaleLabel(SaleLabel { text: promo.text }))
    }

    // Parse misc items
    for item in product.misc {
        // We already parse ingredients/nutritional info earlier.
        if item.header.text == "Ingrediënten" || item.header.text == "Voedingswaarde" {
            continue;
        }
        match item.body {
            Body::Pml { pml_content } => match pml_content.component {
                PmlComponent::Stack(stack) => {
                    for child in stack.children.into_iter() {
                        if child.markdown.contains("Bewaren") {
                            result.additional_items.push(ItemInfo {
                                item_type: ItemType::StorageAdvice,
                                text: child.markdown,
                                text_type: TextType::Markdown,
                            })
                        } else if child.markdown.contains("Land van herkomst") {
                            result.additional_items.push(ItemInfo {
                                item_type: ItemType::CountryOfOrigin,
                                text: child.markdown,
                                text_type: TextType::Markdown,
                            })
                        }
                    }
                }
                PmlComponent::RichText(text) => {
                    if item.header.text == "Bereiding" {
                        result.additional_items.push(ItemInfo {
                            item_type: ItemType::PreparationAdvice,
                            text: text.markdown,
                            text_type: TextType::Markdown,
                        })
                    } else {
                        tracing::debug!(product=?result, "Received unknown RichText in Misc parsing");
                    }
                }
                PmlComponent::Other => {
                    tracing::warn!(product=?result, "Received `Other` content for misc item")
                }
            },

            _ => {
                tracing::warn!(product=?result, "Received `Other` body for misc item")
            }
        }
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
fn parse_picnic_ingredient_blob(blob: &str) -> Option<Vec<IngredientInfo>> {
    // Picnic's ingredient blob is uncharacteristically unstructured, so we have to break it apart ourselves.
    // It has the form "71% tomaat, ui, wortel, 6,6% tomatenpuree (tomatenpuree, zout), etc"

    // The regex we construct is a little unorthodox. Capturing the ingredients between commas directly would require
    // look-ahead/look-behind (within brackets, number with comma separators), not possible with the default `regex` crate.
    // We therefore match all patterns we *don't* want to match first (double nested brackets, brackets, and comma numbers), and then get all normal commas.
    static REGEX: once_cell::sync::Lazy<Regex> =
        once_cell::sync::Lazy::new(|| Regex::new(r#"\([^()]*?(?:\(.*?\))+[^()]*?\)|\(.*?\)|\d+,\d+%|(,)"#).unwrap());

    // Filter all 'normal' commas, they're the only ones in a capture group so it's trivial.
    let comma_indexes = REGEX
        .captures_iter(blob)
        .flat_map(|i| i.get(1))
        .map(|i| i.end())
        .collect_vec();

    if comma_indexes.is_empty() {
        None
    } else {
        // The last item in the list is not followed by a comma, thus we need to manually add one
        let total_ingredients = comma_indexes.len() + 1;

        let mut result = Vec::with_capacity(total_ingredients);

        for i in 0..total_ingredients {
            let ingredient_name = match i {
                0 => blob.split_at(comma_indexes[i] - 1).0,
                _ if i < comma_indexes.len() => &blob[comma_indexes[i - 1]..comma_indexes[i] - 1],
                _ => blob.split_at(comma_indexes[i - 1]).1.trim_end_matches('.'),
            };

            result.push(IngredientInfo {
                name: ingredient_name.trim().to_string(),
            });
        }

        Some(result)
    }
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
    use crate::models::{IngredientInfo, Unit, UnitPrice};
    use crate::providers::picnic_bridge::{
        parse_days_fresh, parse_euro_price, parse_picnic_ingredient_blob, parse_prep_time, parse_unit_price,
    };
    use std::vec;

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
        let ingredient_str = "71% tomaat, ui, wortel, 6,6% tomatenpuree (tomatenpuree, zout), \
        groentebouillonblokje (gejodeerd zout (zout, kaliumjodaat), gedroogde glucosestroop, suiker, \
        groenten (wortel, SELDERIJ, ui, knoflook), zonnebloemolie, gistextract (gistextract, zout), \
        aroma's (SELDERIJ), gedroogde SELDERIJ, water, GERSTEMOUTEXTRACT, kurkuma), \
        knoflook, tijm.";
        let expected = vec![
            IngredientInfo {
                name: "71% tomaat".to_string(),
            },
            IngredientInfo {name: "ui".to_string()},
            IngredientInfo {name: "wortel".to_string()},
            IngredientInfo {
                name: "6,6% tomatenpuree (tomatenpuree, zout)".to_string(),
            },
            IngredientInfo {name: "groentebouillonblokje (gejodeerd zout (zout, kaliumjodaat), gedroogde glucosestroop, suiker, groenten (wortel, SELDERIJ, ui, knoflook), zonnebloemolie, gistextract (gistextract, zout), aroma's (SELDERIJ), gedroogde SELDERIJ, water, GERSTEMOUTEXTRACT, kurkuma)".to_string()},
            IngredientInfo {name: "knoflook".to_string()},
            IngredientInfo {name: "tijm".to_string()}
        ];

        let output = parse_picnic_ingredient_blob(ingredient_str).unwrap();

        assert_eq!(output, expected);
    }
}
