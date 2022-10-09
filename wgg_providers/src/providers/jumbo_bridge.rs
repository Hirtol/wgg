use crate::models::{AllergyTags, AllergyType, FreshLabel, IngredientInfo, ItemInfo, ItemType, NumberOfServings, NutritionalInfo, NutritionalItem, PriceInfo, ProductId, PromotionProduct, SaleDescription, SaleLabel, SaleValidity, SubNutritionalItem, TextType, UnavailableItem, UnavailableReason, UnitPrice, WggDecorator, WggProduct, WggSaleCategory};
use crate::providers::common_bridge::{derive_unit_price, parse_unit_component};
use crate::providers::{common_bridge, ProviderInfo};
use crate::{OffsetPagination, Provider, WggAutocomplete, WggSearchProduct};
use crate::{ProviderError, Result};
use cached::proc_macro::once;
use once_cell::sync::Lazy;
use regex::Regex;
use std::borrow::Cow;
use wgg_jumbo::models::{AvailabilityType, PromotionGroup};
use wgg_jumbo::{BaseApi, BaseJumboApi};

pub(crate) struct JumboBridge {
    pub api: BaseJumboApi,
}

impl JumboBridge {
    pub fn new(api: BaseJumboApi) -> Self {
        JumboBridge { api }
    }
}

#[async_trait::async_trait]
impl ProviderInfo for JumboBridge {
    fn provider(&self) -> Provider {
        Provider::Jumbo
    }

    fn logo_url(&self) -> Cow<'static, str> {
        "https://upload.wikimedia.org/wikipedia/commons/8/8d/Jumbo_Logo.svg".into()
    }

    #[tracing::instrument(name = "jumbo_autocomplete", level="debug", skip_all, fields(query = query))]
    async fn autocomplete(&self, query: &str) -> crate::Result<Vec<WggAutocomplete>> {
        // Cache the response for a day at a time, as the Jumbo autocomplete is just a giant list of terms.
        #[once(time = 86400, result = true)]
        async fn inner(api: &BaseJumboApi) -> Result<Vec<WggAutocomplete>> {
            Ok(api
                .autocomplete()
                .await?
                .autocomplete
                .data
                .into_iter()
                .map(|auto| WggAutocomplete { name: auto })
                .collect())
        }

        let response = inner(&self.api).await?;
        // TODO: Better matching
        Ok(response.into_iter().filter(|c| c.name.contains(query)).collect())
    }

    #[tracing::instrument(name = "jumbo_search", level = "debug", skip(self))]
    async fn search(&self, query: &str, offset: Option<u32>) -> Result<OffsetPagination<WggSearchProduct>> {
        let search_results = self.api.search(query, offset, None).await?;

        #[cfg(feature = "trace-original-api")]
        tracing::trace!("Jumbo Search: {:#?}", search_results);

        Ok(OffsetPagination {
            items: search_results
                .products
                .data
                .into_iter()
                .map(parse_jumbo_item_to_search_item)
                .collect(),
            total_items: search_results.products.total as usize,
            offset: search_results.products.offset,
        })
    }

    async fn product(&self, product_id: &str) -> Result<WggProduct> {
        let result = self.api.product(&product_id.parse()?).await?;

        #[cfg(feature = "trace-original-api")]
        tracing::trace!("Jumbo Product: {:#?}", result);

        Ok(parse_jumbo_product_to_crate_product(result.product.data))
    }

    async fn promotions(&self) -> Result<Vec<WggSaleCategory>> {
        let tab_ids = self.api.promotion_tabs().await?;

        let all_proms = tab_ids
            .tabs
            .iter()
            .find(|tab| tab.id == "alle")
            .ok_or(ProviderError::NothingFound)?;
        let current_runtime = all_proms.runtimes.first().ok_or(ProviderError::NothingFound)?;

        let result = self
            .api
            .promotion_group(&all_proms.id, &current_runtime.id, None, None)
            .await?;

        #[cfg(feature = "trace-original-api")]
        tracing::trace!("Jumbo Promotions: {:#?}", result);

        parse_jumbo_promotions(result)
    }

    async fn promotions_sublist(&self, sublist_id: &str) -> Result<OffsetPagination<WggSearchProduct>> {
        let promo_id = sublist_id.parse()?;
        let result = self.api.products_promotion(Some(&promo_id), 100, 0).await?.products;

        #[cfg(feature = "trace-original-api")]
        tracing::trace!("Jumbo Promotion Products: {:#?}", result);

        Ok(OffsetPagination {
            items: result.data.into_iter().map(parse_jumbo_item_to_search_item).collect(),
            total_items: result.total as usize,
            offset: result.offset,
        })
    }
}

fn parse_jumbo_promotions(promotion: PromotionGroup) -> Result<Vec<WggSaleCategory>> {
    let result = promotion
        .categories
        .into_iter()
        .flat_map(|item| item.promotions)
        .map(|item| {
            let mut result = WggSaleCategory {
                id: item.id.into(),
                name: item.title,
                image_urls: vec![item.image.url],
                limited_items: item
                    .products
                    .into_iter()
                    .map(|product| PromotionProduct::ProductId(ProductId { id: product.into() }))
                    .collect(),
                decorators: vec![],
                provider: Provider::Jumbo,
            };

            if let Some(sub) = item.subtitle {
                result
                    .decorators
                    .push(WggDecorator::SaleDescription(SaleDescription { text: sub }));
            }

            if let Some(tag) = item.tags.into_iter().next() {
                result.decorators.push(WggDecorator::SaleLabel(SaleLabel { text: tag }));
            }

            result.decorators.push(WggDecorator::SaleValidity(SaleValidity {
                valid_from: item.start_date,
                valid_until: item.end_date,
            }));

            result
        })
        .collect();

    Ok(result)
}

fn parse_jumbo_product_to_crate_product(mut product: wgg_jumbo::models::Product) -> WggProduct {
    let mut result = WggProduct {
        id: product.id.into(),
        name: product.title,
        description: product
            .regulated_title
            .map(|title| format!("{}\n{}", title, product.details_text.as_deref().unwrap_or_default()))
            .or(product.details_text)
            .unwrap_or_default(),
        price_info: PriceInfo {
            display_price: product
                .prices
                .promotional_price
                .map(|price| price.amount)
                .unwrap_or(product.prices.price.amount),
            original_price: product.prices.price.amount,
            // Will be parsed
            unit_price: None
        },
        unit_quantity: product
            .quantity
            .as_deref()
            .and_then(common_bridge::parse_quantity)
            .unwrap_or_default(),
        available: product.available,
        image_urls: product.image_info.primary_view.into_iter().map(|i| i.url).collect(),
        // Will be parsed
        ingredients: vec![],
        // Will be parsed
        nutritional: None,
        // Will be parsed
        allergy_info: vec![],
        // Will be parsed
        additional_items: vec![],
        // Will be parsed
        decorators: vec![],
        provider: Provider::Jumbo,
    };

    // Unit Pricing
    if let Some(price) = product.prices.unit_price {
        if let Some(unit) = parse_unit_component(&price.unit) {
            result.price_info.unit_price = UnitPrice {
                unit,
                price: price.price.amount,
            }
            .into()
        }
    } else {
        result.price_info.unit_price = derive_unit_price(&result.unit_quantity, result.price_info.display_price);
    }

    // Promotions
    if let Some(promotion) = product.promotion {
        result.decorators.extend(
            promotion
                .tags
                .into_iter()
                .map(|t| SaleLabel { text: t.text })
                .map(WggDecorator::SaleLabel),
        );

        result.decorators.push(WggDecorator::SaleValidity(SaleValidity {
            valid_from: promotion.from_date,
            valid_until: promotion.to_date,
        }));
    }

    // Availability
    if product.availability.availability != AvailabilityType::Available {
        let unavailable = UnavailableItem {
            reason: match product.availability.availability {
                AvailabilityType::TemporarilyUnavailable => UnavailableReason::TemporarilyUnavailable,
                AvailabilityType::Unavailable => UnavailableReason::OutOfAssortment,
                _ => UnavailableReason::Unknown,
            },
            explanation_short: product.availability.label,
            explanation_long: product.availability.reason,
            replacements: Vec::new(),
        };

        result.decorators.push(WggDecorator::Unavailable(unavailable));
    }

    // Freshness
    if let Some(fresh) = product.badge_description.as_deref().and_then(parse_badge_description) {
        result.decorators.push(WggDecorator::FreshLabel(fresh))
    }

    // Ingredients
    if let Some(primary_ingredients) = product.ingredient_info.pop() {
        result.ingredients = primary_ingredients
            .ingredients
            .into_iter()
            .map(|item| IngredientInfo { name: item.name })
            .collect();
    }

    // Nutritional
    if let Some(primary_nutrition) = product.nutritional_information.first() {
        result.nutritional = NutritionalInfo {
            info_unit: "per 100g".to_string(),
            items: parse_nutritional_info(primary_nutrition),
        }
        .into()
    }

    // Allergies
    if let Some(text) = product.allergy_text {
        result.allergy_info = parse_allergy_info(&text);
    }

    // Servings
    if let Some(servings) = product.number_of_servings {
        if let Ok(amount) = servings.parse() {
            result
                .decorators
                .push(WggDecorator::NumberOfServings(NumberOfServings { amount }));
        }
    }

    // Additional items
    if let Some(usage_and_safety) = product.usage_and_safety_info {
        if let Some(prep_advice) = usage_and_safety.preparation_and_usage {
            result.additional_items.push(ItemInfo {
                item_type: ItemType::PreparationAdvice,
                text: prep_advice,
                text_type: TextType::PlainText
            })
        }

        if let Some(storage) = usage_and_safety.storage_type {
            result.additional_items.push(ItemInfo {
                item_type: ItemType::StorageAdvice,
                text: storage,
                text_type: TextType::PlainText
            })
        }

        if let Some(safety) = usage_and_safety.safety_warning {
            result.additional_items.push(ItemInfo {
                item_type: ItemType::SafetyWarning,
                text: safety,
                text_type: TextType::PlainText
            })
        }
    }

    if let Some(origin) = product.origin_info {
        if let Some(origin) = origin.country_of_origin.or(origin.fishing_area) {
            result.additional_items.push(ItemInfo {
                item_type: ItemType::CountryOfOrigin,
                text: origin,
                text_type: TextType::PlainText
            })
        }
    }

    result
}

/// Parse a full picnic [wgg_jumbo::models::SingleArticle] to our normalised [SearchItem]
fn parse_jumbo_item_to_search_item(article: wgg_jumbo::models::PartialProduct) -> WggSearchProduct {
    let mut result = WggSearchProduct {
        id: article.id.into(),
        name: article.title,
        full_price: article.prices.price.amount,
        display_price: article
            .prices
            .promotional_price
            .map(|price| price.amount)
            .unwrap_or(article.prices.price.amount),
        unit_quantity: article
            .quantity
            .as_deref()
            .and_then(common_bridge::parse_quantity)
            .unwrap_or_default(),
        unit_price: None,
        available: article.available,
        image_url: article.image_info.primary_view.first().map(|i| i.url.clone()),
        decorators: Vec::new(),
        provider: Provider::Jumbo,
    };

    // Unit Pricing
    if let Some(price) = article.prices.unit_price {
        if let Some(unit) = parse_unit_component(&price.unit) {
            result.unit_price = UnitPrice {
                unit,
                price: price.price.amount,
            }
            .into()
        }
    } else {
        result.unit_price = derive_unit_price(&result.unit_quantity, result.display_price);
    }

    // Promotions
    if let Some(promotion) = article.promotion {
        result.decorators.extend(
            promotion
                .tags
                .into_iter()
                .map(|t| SaleLabel { text: t.text })
                .map(WggDecorator::SaleLabel),
        );
        result.decorators.push(WggDecorator::SaleValidity(SaleValidity {
            valid_from: promotion.from_date,
            valid_until: promotion.to_date,
        }));
    }

    // Availability
    if article.availability.availability != AvailabilityType::Available {
        let unavailable = UnavailableItem {
            reason: match article.availability.availability {
                AvailabilityType::TemporarilyUnavailable => UnavailableReason::TemporarilyUnavailable,
                AvailabilityType::Unavailable => UnavailableReason::OutOfAssortment,
                _ => UnavailableReason::Unknown,
            },
            explanation_short: article.availability.label,
            explanation_long: article.availability.reason,
            replacements: Vec::new(),
        };

        result.decorators.push(WggDecorator::Unavailable(unavailable));
    }

    // Freshness
    if let Some(fresh) = article.badge_description.as_deref().and_then(parse_badge_description) {
        result.decorators.push(WggDecorator::FreshLabel(fresh))
    }

    result
}

/// Parse the Jumbo `badge_description` element, which frequently contains a [FreshLabel] decorator.
///
/// Format expected: `7+ dagen vers`
fn parse_badge_description(badge_desc: &str) -> Option<FreshLabel> {
    if badge_desc.contains("dagen vers") {
        let days_fresh = badge_desc.split_whitespace().next()?;
        // Filter out the `+`
        let days_fresh = days_fresh.trim_end_matches('+').parse().ok()?;

        let fresh_label = FreshLabel { days_fresh };

        Some(fresh_label)
    } else {
        None
    }
}

/// Parse nutritional information of part of the product.
///
/// Will try to ensure related entries are grouped in a parent/child relation, f.e, fats & saturated fats.
fn parse_nutritional_info(nut_info: &wgg_jumbo::models::NutritionalInformation) -> Vec<NutritionalItem> {
    let mut nut_result = vec![];

    let len = nut_info.nutritional_data.entries.len();
    let mut i = 0;

    while i < len {
        let parent_item = &nut_info.nutritional_data.entries[i];
        let mut to_add = NutritionalItem {
            name: parent_item.name.clone(),
            value: parent_item.value_per100g.clone(),
            sub_values: vec![],
        };

        // Not the last element
        if i < len - 1 {
            let next_i = i + 1;
            // Perform a bounded look-ahead to gather sub-items for nutrition
            // This is mainly related to entries for fat/carbohydrates where you have a 'parent' item such as "Fats"
            // and a child item such as "Saturated Fat".
            for j in next_i..len {
                let current_item = &nut_info.nutritional_data.entries[j];
                if current_item.name.is_empty() || current_item.name.contains("Waarvan") {
                    to_add.sub_values.push(SubNutritionalItem {
                        name: current_item.name.clone(),
                        value: current_item.value_per100g.clone(),
                    })
                } else {
                    // Skip the previous items
                    i = j - 1;
                    break;
                }
            }
        }

        nut_result.push(to_add);
        i += 1;
    }

    nut_result
}

/// Parse the allergy text blob from Jumbo.
///
/// Simple list similar to `Bevat Selderij,Kan het volgende bevatten Eieren,etc`
fn parse_allergy_info(allergy_text: &str) -> Vec<AllergyTags> {
    static ALLERGY_RX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"[Bb]evat(ten)? (\w+)"#).expect("Could not compile regex"));

    ALLERGY_RX
        .captures_iter(allergy_text)
        .flat_map(|cap| cap.get(2).map(|i| (i, cap.get(1))))
        .map(|(item, may_contain)| AllergyTags {
            name: item.as_str().to_string(),
            contains: {
                if may_contain.is_some() {
                    AllergyType::MayContain
                } else {
                    AllergyType::Contains
                }
            },
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::models::{AllergyTags, AllergyType, FreshLabel};
    use crate::providers::jumbo_bridge::{parse_allergy_info, parse_badge_description};

    #[test]
    pub fn test_parse_badge_description() {
        let examples = vec!["7+ dagen vers", "2+ dagen vers", "20+ dagen vers"];
        let expected = vec![
            FreshLabel { days_fresh: 7 },
            FreshLabel { days_fresh: 2 },
            FreshLabel { days_fresh: 20 },
        ];

        assert_eq!(
            examples
                .into_iter()
                .flat_map(parse_badge_description)
                .collect::<Vec<_>>(),
            expected
        );
    }

    #[test]
    pub fn test_allergy_text() {
        let example = "Bevat Selderij,\
        Kan het volgende bevatten Eieren,\
        Kan het volgende bevatten Melk,\
        Kan het volgende bevatten Mosterd,\
        Kan het volgende bevatten Tarwe,\
        Kan het volgende bevatten Gluten,\
        Kan het volgende bevatten Soja";

        let expected = vec![
            AllergyTags {
                name: "Selderij".to_string(),
                contains: AllergyType::Contains,
            },
            AllergyTags {
                name: "Eieren".to_string(),
                contains: AllergyType::MayContain,
            },
            AllergyTags {
                name: "Melk".to_string(),
                contains: AllergyType::MayContain,
            },
            AllergyTags {
                name: "Mosterd".to_string(),
                contains: AllergyType::MayContain,
            },
            AllergyTags {
                name: "Tarwe".to_string(),
                contains: AllergyType::MayContain,
            },
            AllergyTags {
                name: "Gluten".to_string(),
                contains: AllergyType::MayContain,
            },
            AllergyTags {
                name: "Soja".to_string(),
                contains: AllergyType::MayContain,
            },
        ];

        assert_eq!(parse_allergy_info(example), expected);
    }
}
