use crate::error::Result;
use crate::models::sale_types::SaleType;
use crate::models::{
    AllergyTags, AllergyType, Description, FreshLabel, IngredientInfo, ItemInfo, ItemType, NumberOfServings,
    NutritionalInfo, NutritionalItem, PriceInfo, ProductIdT, Provider, ProviderMetadata, SaleInformation,
    SaleResolutionStrategy, SaleValidity, SubNutritionalItem, TextType, UnavailableItem, UnavailableReason, UnitPrice,
    WggAutocomplete, WggDecorator, WggProduct, WggSaleCategory, WggSaleGroupComplete, WggSaleGroupLimited, WggSaleItem,
    WggSearchProduct,
};
use crate::pagination::OffsetPagination;
use crate::providers::common_bridge::{derive_unit_price, parse_sale_label, parse_unit_component};
use crate::providers::{ProviderInfo, StaticProviderInfo, common_bridge};
use crate::{ProviderError, lazy_re};
use anyhow::Context;
use cached::proc_macro::once;
use regex::Regex;
use wgg_jumbo::models::{AvailabilityType, PromotionGroupContent};
use wgg_jumbo::{BaseApi, BaseJumboApi};

pub(crate) struct JumboBridge {
    pub api: BaseJumboApi,
}

impl JumboBridge {
    pub fn new(api: BaseJumboApi) -> Self {
        JumboBridge { api }
    }
}

impl StaticProviderInfo for JumboBridge {
    fn provider() -> Provider {
        Provider::Jumbo
    }

    fn metadata() -> ProviderMetadata {
        ProviderMetadata {
            logo_url: "https://upload.wikimedia.org/wikipedia/commons/8/8d/Jumbo_Logo.svg".into(),
            sale_strategy: SaleResolutionStrategy::Pessimistic,
            supports_cart: false,
        }
    }
}

#[async_trait::async_trait]
impl ProviderInfo for JumboBridge {
    fn provider(&self) -> Provider {
        <Self as StaticProviderInfo>::provider()
    }

    fn metadata(&self) -> ProviderMetadata {
        <Self as StaticProviderInfo>::metadata()
    }

    #[tracing::instrument(name = "jumbo_autocomplete", level="debug", skip_all, fields(query = query))]
    async fn autocomplete(&self, query: &str) -> crate::error::Result<Vec<WggAutocomplete>> {
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
            .find(|tab| tab.id == "actieprijs")
            .ok_or(ProviderError::NothingFound)?;
        let current_runtime = all_proms.runtimes.first().ok_or(ProviderError::NothingFound)?;

        let result = self
            .api
            .promotion_content(&all_proms.id, &current_runtime.id, None, None)
            .await?;

        #[cfg(feature = "trace-original-api")]
        tracing::trace!("Jumbo Promotions: {:#?}", result);

        let main_sale_items = result
            .groups
            .into_iter()
            .find(|group| group.title.contains("gangpad"))
            .ok_or(ProviderError::NothingFound)?;
        let PromotionGroupContent::Categories(categories) = main_sale_items.content else {
            return Err(anyhow::anyhow!(
                "Found promotion group did not contain categories as expected!"
            ))?;
        };

        Ok(categories
            .into_iter()
            .map(|item| WggSaleCategory {
                id: None,
                name: item.title,
                provider: Provider::Jumbo,
                items: item
                    .promotions
                    .into_iter()
                    .flat_map(parse_jumbo_promotion)
                    .map(WggSaleItem::Group)
                    .collect(),
                image_urls: Vec::new(),
                complete: true,
            })
            .collect())
    }

    async fn promotions_sublist(&self, sublist_id: &str) -> Result<WggSaleGroupComplete> {
        let promo_id = sublist_id.parse()?;
        let promotion = self.api.promotion(&promo_id);
        let result = self.api.products_promotion(Some(&promo_id), 100, 0);

        let (promotion, product_list) = futures::try_join!(promotion, result)?;

        #[cfg(feature = "trace-original-api")]
        tracing::trace!("Jumbo Promotion Products: {:#?}", product_list);

        parse_jumbo_promotion_complete(promotion, product_list)
    }
}

fn parse_jumbo_promotion(promotion: wgg_jumbo::models::Promotion) -> Option<WggSaleGroupLimited> {
    Some(WggSaleGroupLimited {
        id: promotion.id.clone().into(),
        name: promotion.title,
        image_urls: promotion.image.map(|img| vec![img.url]).unwrap_or_default(),
        items: promotion
            .products
            .into_iter()
            .map(|product| ProductIdT { id: product.into() })
            .collect(),
        // Jumbo has a tendency to include not on-sale items in their sale listings (Ronde prijs! kind of promotions)
        // Those have no `tags` and will thus fail here.
        sale_info: parse_promotion_to_sale_info(
            Some(promotion.id.into()),
            &promotion.tags,
            SaleValidity {
                valid_from: promotion.start_date,
                valid_until: promotion.end_date,
            },
        )?,
        sale_description: promotion.subtitle,
        provider: Provider::Jumbo,
    })
}

fn parse_jumbo_promotion_complete(
    promotion: wgg_jumbo::models::Promotion,
    product_list: wgg_jumbo::models::ProductList,
) -> Result<WggSaleGroupComplete> {
    let promo = parse_jumbo_promotion(promotion).context("Provided list is not a true sale (may be a faux-sale!)")?;

    let items = product_list
        .products
        .data
        .into_iter()
        .map(parse_jumbo_item_to_search_item)
        .map(|mut item| {
            // As it turns out Jumbo doesn't embed the sale info into the individual sale items,
            // we have to do this ourselves!.
            item.sale_information = Some(promo.sale_info.clone());

            item
        })
        .collect();

    Ok(WggSaleGroupComplete {
        id: promo.id,
        name: promo.name,
        image_urls: promo.image_urls,
        items,
        sale_info: promo.sale_info,
        sale_description: promo.sale_description,
        provider: promo.provider,
    })
}

fn parse_jumbo_product_to_crate_product(mut product: wgg_jumbo::models::Product) -> WggProduct {
    let mut result = WggProduct {
        id: product.id.into(),
        name: product.title,
        description: {
            let text = product
                .regulated_title
                .map(|title| format!("{}\n{}", title, product.details_text.as_deref().unwrap_or_default()))
                .or(product.details_text)
                .unwrap_or_default();

            Description {
                text,
                text_type: TextType::PlainText,
            }
        },
        price_info: PriceInfo {
            display_price: product
                .prices
                .promotional_price
                .map(|price| price.amount)
                .unwrap_or(product.prices.price.amount),
            original_price: product.prices.price.amount,
            // Will be parsed
            unit_price: None,
        },
        unit_quantity: product
            .quantity
            .as_deref()
            .and_then(common_bridge::parse_quantity)
            .unwrap_or_default(),
        // Will be parsed
        unavailable_details: None,
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
        // Will be parsed
        sale_information: None,
        provider: Provider::Jumbo,
    };

    // Unit Pricing
    match product.prices.unit_price {
        Some(wgg_jumbo::models::UnitPrice {
            unit,
            price: Some(amount),
        }) => {
            if let Some(unit) = parse_unit_component(&unit) {
                result.price_info.unit_price = UnitPrice {
                    unit,
                    price: amount.amount,
                }
                .into()
            }
        }
        _ => {
            result.price_info.unit_price = derive_unit_price(&result.unit_quantity, result.price_info.display_price);
        }
    }

    // Promotions
    if let Some(promotion) = product.promotion {
        result.sale_information = parse_promotion_to_sale_info(
            Some(promotion.id),
            &promotion.tags,
            SaleValidity {
                valid_from: promotion.from_date,
                valid_until: promotion.to_date,
            },
        );
    }

    // Availability
    if product.availability.availability != AvailabilityType::Available {
        result.unavailable_details = UnavailableItem {
            reason: match product.availability.availability {
                AvailabilityType::TemporarilyUnavailable => UnavailableReason::TemporarilyUnavailable,
                AvailabilityType::Unavailable => UnavailableReason::OutOfAssortment,
                _ => UnavailableReason::Unknown,
            },
            explanation_short: product.availability.label,
            explanation_long: product.availability.reason,
            replacements: Vec::new(),
        }
        .into();
    } else if !product.available {
        tracing::warn!(
            product_id=?result.id,
            "Jumbo bridge detected a product which is indicated as `Available`, but isn't!"
        );
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
                text_type: TextType::PlainText,
            })
        }

        if let Some(storage) = usage_and_safety.storage_type {
            result.additional_items.push(ItemInfo {
                item_type: ItemType::StorageAdvice,
                text: storage,
                text_type: TextType::PlainText,
            })
        }

        if let Some(safety) = usage_and_safety.safety_warning {
            result.additional_items.push(ItemInfo {
                item_type: ItemType::SafetyWarning,
                text: safety,
                text_type: TextType::PlainText,
            })
        }
    }

    if let Some(origin) = product.origin_info {
        if let Some(origin) = origin.country_of_origin.or(origin.fishing_area) {
            result.additional_items.push(ItemInfo {
                item_type: ItemType::CountryOfOrigin,
                text: origin,
                text_type: TextType::PlainText,
            })
        }
    }

    result
}

/// Parse a full picnic [wgg_jumbo::models::SingleArticle] to our normalised [SearchItem]
fn parse_jumbo_item_to_search_item(mut article: wgg_jumbo::models::PartialProduct) -> WggSearchProduct {
    let mut result = WggSearchProduct {
        id: article.id.into(),
        name: article.title,
        unit_quantity: article
            .quantity
            .as_deref()
            .and_then(common_bridge::parse_quantity)
            .unwrap_or_default(),
        image_url: article
            .image_info
            .take()
            .map(|info| info.primary_view.into_iter())
            .and_then(|it| it.map(|i| i.url).next()),
        decorators: Vec::new(),
        sale_information: None,
        provider: Provider::Jumbo,
        price_info: PriceInfo {
            original_price: article.prices.price.amount,
            display_price: article
                .prices
                .promotional_price
                .map(|price| price.amount)
                .unwrap_or(article.prices.price.amount),
            unit_price: None,
        },
        unavailable_details: None,
    };

    // Unit Pricing
    match article.prices.unit_price {
        Some(wgg_jumbo::models::UnitPrice {
            unit,
            price: Some(amount),
        }) => {
            if let Some(unit) = parse_unit_component(&unit) {
                result.price_info.unit_price = UnitPrice {
                    unit,
                    price: amount.amount,
                }
                .into()
            }
        }
        _ => {
            result.price_info.unit_price = derive_unit_price(&result.unit_quantity, result.price_info.display_price);
        }
    }

    // Promotions
    if let Some(promotion) = article.promotion {
        result.sale_information = parse_promotion_to_sale_info(
            Some(promotion.id),
            &promotion.tags,
            SaleValidity {
                valid_from: promotion.from_date,
                valid_until: promotion.to_date,
            },
        );
    }

    // Availability
    if article.availability.availability != AvailabilityType::Available {
        result.unavailable_details = UnavailableItem {
            reason: match article.availability.availability {
                AvailabilityType::TemporarilyUnavailable => UnavailableReason::TemporarilyUnavailable,
                AvailabilityType::Unavailable => UnavailableReason::OutOfAssortment,
                _ => UnavailableReason::Unknown,
            },
            explanation_short: article.availability.label,
            explanation_long: article.availability.reason,
            replacements: Vec::new(),
        }
        .into();
    } else if !article.available {
        tracing::warn!(
            product_id=?result.id,
            "Jumbo bridge detected a product which is indicated as `Available`, but isn't!"
        );
    }

    // Freshness
    if let Some(fresh) = article.badge_description.as_deref().and_then(parse_badge_description) {
        result.decorators.push(WggDecorator::FreshLabel(fresh))
    }

    result
}

/// Parse a [ProductPromotion] to a [SaleInformation] type.
fn parse_promotion_to_sale_info(
    promotion_id: Option<String>,
    tags: &[impl AsRef<str>],
    sale_validity: SaleValidity,
) -> Option<SaleInformation> {
    // Get the first valid label
    let parsed_label = tags
        .iter()
        .flat_map(|tag| Some((tag, parse_sale_label(tag.as_ref())?)))
        .next();
    let to_sale = |sale_type: Option<SaleType>, label: &str| SaleInformation {
        id: promotion_id,
        label: label.to_string(),
        additional_label: tags
            .iter()
            .filter(|t| t.as_ref() != label)
            .map(|t| t.as_ref().to_string())
            .collect(),
        sale_validity,
        sale_type,
    };

    if let Some((tag, sale_type)) = parsed_label {
        Some(to_sale(Some(sale_type), tag.as_ref()))
    } else {
        // If we couldn't parse a valid label we'll have to make a best effort delivery.
        // Try and avoid the `Online only` and `Voor bezorging` tags.
        let label = tags
            .iter()
            .find(|tag| !tag.as_ref().contains("online") && !tag.as_ref().contains("bezorg"))
            .or_else(|| tags.first());

        label.map(|label| to_sale(None, label.as_ref()))
    }
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
    lazy_re!(ALLERGY_RX, r"[Bb]evat(ten)? (\w+)");

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
