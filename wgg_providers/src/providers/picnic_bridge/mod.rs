use crate::models::{
    AllergyTags, AllergyType, CentPrice, Description, FreshLabel, IngredientInfo, ItemInfo, ItemType, MoreButton,
    NutritionalInfo, NutritionalItem, PrepTime, PriceInfo, SaleLabel, SaleValidity, SubNutritionalItem, TextType,
    UnavailableItem, UnitPrice, WggDecorator, WggProduct, WggSaleCategory, WggSaleGroupComplete, WggSaleItem,
};
use crate::providers::common_bridge::parse_quantity;
use crate::providers::{common_bridge, ProviderInfo, StaticProviderInfo};
use crate::{OffsetPagination, Provider, ProviderError, WggAutocomplete, WggSearchProduct};
use chrono::{Datelike, LocalResult, NaiveDate, TimeZone};
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Jitter, Quota};
use itertools::Itertools;
use regex::Regex;
use secrecy::ExposeSecret;
use std::borrow::Cow;
use std::num::NonZeroU32;
use std::time::Duration;
use wgg_picnic::models::{Body, Decorator, ImageSize, PmlComponent, SubCategory, UnavailableReason};
use wgg_picnic::PicnicApi;

use crate::Result;

mod authentication;

pub use authentication::PicnicCredentials;

pub const PICNIC_RECOMMENDED_RPS: Option<NonZeroU32> = NonZeroU32::new(5);
const JITTER: Duration = Duration::from_millis(500);

/// A separate bridge struct to allow for easier caching.
pub(crate) struct PicnicBridge {
    pub api: tokio::sync::RwLock<PicnicApi>,
    limiter: governor::RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
    credentials: PicnicCredentials,
    /// Lock used to ensure only *one* future tries to refresh the auth-token when it is expired.
    refresh_lock: tokio::sync::Mutex<()>,
}

impl PicnicBridge {
    pub(crate) async fn new(credentials: PicnicCredentials, limit_rps: Option<NonZeroU32>) -> Result<Self> {
        let config = Default::default();
        let api = if let Some(auth_token) = credentials.to_credentials() {
            PicnicApi::new(auth_token, config)
        } else {
            PicnicApi::from_login(credentials.email(), credentials.password().expose_secret(), config).await?
        };

        tracing::trace!(auth_token=?api.credentials().auth_token, "Picnic Login Complete");

        Ok(Self::from_api(api, credentials, limit_rps))
    }

    pub(crate) fn from_api(api: PicnicApi, credentials: PicnicCredentials, limit_rps: Option<NonZeroU32>) -> Self {
        let limiter =
            governor::RateLimiter::direct(Quota::per_second(limit_rps.unwrap_or(PICNIC_RECOMMENDED_RPS.unwrap())));

        PicnicBridge {
            api: api.into(),
            limiter,
            credentials,
            refresh_lock: Default::default(),
        }
    }

    pub(crate) fn credentials(&self) -> &PicnicCredentials {
        &self.credentials
    }

    async fn wait_rate_limit(&self) {
        // Only use Jitter if we have to start delaying calls.
        if self.limiter.check().is_err() {
            self.limiter.until_ready_with_jitter(Jitter::up_to(JITTER)).await
        }
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

// These implementations contain a lot of duplicate code to handle the Auth token refresh (when needed).
// Unfortunately, extracting these out to a common function taking a closure required higher-lifetime bounds I couldn't get to work (even with boxing :/)
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
        self.wait_rate_limit().await;
        let api_result = self.api.read().await.suggestions(query).await;
        let result = match api_result {
            Ok(res) => res,
            Err(wgg_picnic::ApiError::AuthError) => {
                if self.refresh_lock.try_lock().is_ok() {
                    tracing::info!("Refreshing Picnic authentication token due to failure");
                    let mut api_guard = self.api.write().await;

                    api_guard
                        .login(self.credentials.email(), self.credentials.password().expose_secret())
                        .await?;

                    api_guard.suggestions(query).await?
                } else {
                    // Once the refresh_lock is released we will *eventually* be able to lock it here and re-execute the request.
                    // If it fails again just error out.
                    let _ = self.refresh_lock.lock().await;
                    self.api.read().await.suggestions(query).await?
                }
            }
            Err(e) => return Err(e.into()),
        };

        #[cfg(feature = "trace-original-api")]
        tracing::trace!("Picnic Autocomplete: {:#?}", result);

        Ok(result
            .into_iter()
            .map(|i| WggAutocomplete { name: i.suggestion })
            .collect())
    }

    #[tracing::instrument(name = "picnic_search", level = "debug", skip(self))]
    async fn search(&self, query: &str, offset: Option<u32>) -> Result<OffsetPagination<WggSearchProduct>> {
        self.wait_rate_limit().await;
        let api_result = self.api.read().await.search(query).await;
        let result = match api_result {
            Ok(res) => res,
            Err(wgg_picnic::ApiError::AuthError) => {
                if self.refresh_lock.try_lock().is_ok() {
                    tracing::info!("Refreshing Picnic authentication token due to failure");
                    let mut api_guard = self.api.write().await;

                    api_guard
                        .login(self.credentials.email(), self.credentials.password().expose_secret())
                        .await?;

                    api_guard.search(query).await?
                } else {
                    // Once the refresh_lock is released we will *eventually* be able to lock it here and re-execute the request.
                    // If it fails again just error out.
                    let _ = self.refresh_lock.lock().await;
                    self.api.read().await.search(query).await?
                }
            }
            Err(e) => return Err(e.into()),
        };
        let offset = offset.unwrap_or_default();

        #[cfg(feature = "trace-original-api")]
        tracing::trace!("Picnic Search: {:#?}", result);

        let result: Vec<WggSearchProduct> = result
            .into_iter()
            .flat_map(|res| {
                res.items.into_iter().filter_map(|item| {
                    if let wgg_picnic::models::SearchItem::SingleArticle(article) = item {
                        Some(parse_picnic_item_to_search_item(article))
                    } else {
                        None
                    }
                })
            })
            .skip(offset as usize)
            .collect();

        let offset = OffsetPagination {
            total_items: result.len() + offset as usize,
            items: result,
            offset,
        };

        Ok(offset)
    }

    async fn product(&self, product_id: &str) -> Result<WggProduct> {
        self.wait_rate_limit().await;
        let api_result = self.api.read().await.product(product_id).await;
        let result = match api_result {
            Ok(res) => res,
            Err(wgg_picnic::ApiError::AuthError) => {
                if self.refresh_lock.try_lock().is_ok() {
                    tracing::info!("Refreshing Picnic authentication token due to failure");
                    let mut api_guard = self.api.write().await;

                    api_guard
                        .login(self.credentials.email(), self.credentials.password().expose_secret())
                        .await?;

                    api_guard.product(product_id).await?
                } else {
                    // Once the refresh_lock is released we will *eventually* be able to lock it here and re-execute the request.
                    // If it fails again just error out.
                    let _ = self.refresh_lock.lock().await;
                    self.api.read().await.product(product_id).await?
                }
            }
            Err(e) => return Err(e.into()),
        };

        #[cfg(feature = "trace-original-api")]
        tracing::trace!("Picnic Product: {:#?}", result);

        parse_picnic_full_product_to_product(result)
    }

    async fn promotions(&self) -> Result<Vec<WggSaleCategory>> {
        self.wait_rate_limit().await;
        let api_result = self.api.read().await.promotions(None, 1).await;
        let result = match api_result {
            Ok(res) => res,
            Err(wgg_picnic::ApiError::AuthError) => {
                if self.refresh_lock.try_lock().is_ok() {
                    tracing::info!("Refreshing Picnic authentication token due to failure");
                    let mut api_guard = self.api.write().await;

                    api_guard
                        .login(self.credentials.email(), self.credentials.password().expose_secret())
                        .await?;

                    api_guard.promotions(None, 1).await?
                } else {
                    // Once the refresh_lock is released we will *eventually* be able to lock it here and re-execute the request.
                    // If it fails again just error out.
                    let _ = self.refresh_lock.lock().await;
                    self.api.read().await.promotions(None, 1).await?
                }
            }
            Err(e) => return Err(e.into()),
        };

        #[cfg(feature = "trace-original-api")]
        tracing::trace!("Picnic Promotions: {:#?}", result);

        Ok(result.into_iter().flat_map(parse_picnic_promotion).collect())
    }

    async fn promotions_sublist(&self, sublist_id: &str) -> Result<WggSaleGroupComplete> {
        self.wait_rate_limit().await;
        // When querying for a sublist with `depth > 1` we just get a raw array of SingleArticles
        let api_result = self.api.read().await.promotions(Some(sublist_id), 0).await;
        let result = match api_result {
            Ok(res) => res,
            Err(wgg_picnic::ApiError::AuthError) => {
                if self.refresh_lock.try_lock().is_ok() {
                    tracing::info!("Refreshing Picnic authentication token due to failure");
                    let mut api_guard = self.api.write().await;

                    api_guard
                        .login(self.credentials.email(), self.credentials.password().expose_secret())
                        .await?;

                    api_guard.promotions(Some(sublist_id), 0).await?
                } else {
                    // Once the refresh_lock is released we will *eventually* be able to lock it here and re-execute the request.
                    // If it fails again just error out.
                    let _ = self.refresh_lock.lock().await;
                    self.api.read().await.promotions(Some(sublist_id), 0).await?
                }
            }
            Err(e) => return Err(e.into()),
        };

        #[cfg(feature = "trace-original-api")]
        tracing::trace!("Picnic Promotions Sublist: {:#?}", result);

        let sale_cat = result
            .into_iter()
            .flat_map(parse_picnic_promotion)
            .next()
            .ok_or(ProviderError::NothingFound)?;

        Ok(WggSaleGroupComplete {
            id: sale_cat
                .id
                .expect("Picnic Category does not have an id when it is expected"),
            name: sale_cat.name,
            image_urls: sale_cat.image_urls,
            items: sale_cat
                .items
                .into_iter()
                .flat_map(|item| match item {
                    WggSaleItem::Product(product) => Some(product),
                    _ => {
                        tracing::warn!(?item, "Sublist completion was provided a non-complete SearchProduct!");
                        None
                    }
                })
                .collect(),
            decorators: Vec::new(),
            provider: sale_cat.provider,
        })
    }
}

fn parse_picnic_promotion(promotion: SubCategory) -> Option<WggSaleCategory> {
    let SubCategory::Category(category) = promotion else {
        tracing::warn!(?promotion, "Expected category for promotion parsing, but found other");
        return None;
    };

    let mut result = WggSaleCategory {
        id: Some(category.id),
        name: category.name,
        image_urls: vec![],
        items: vec![],
        provider: Provider::Picnic,
        complete: true,
    };

    // Decorators
    for dec in category.decorators {
        if let Decorator::MoreButton { images, .. } = &dec {
            result.image_urls = images
                .iter()
                .map(|id| wgg_picnic::images::image_url(id, ImageSize::Medium))
                .collect();

            result.complete = false;
        } else {
            tracing::debug!(?dec, "Unknown decorator encountered in parsing Picnic sale")
        }
    }

    result.items = category
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
        .map(parse_picnic_item_to_search_item)
        .map(WggSaleItem::Product)
        .collect();

    Some(result)
}

/// Parse a full picnic [wgg_picnic::models::ProductDetails] to our normalised [Product]
fn parse_picnic_full_product_to_product(product: wgg_picnic::models::ProductArticle) -> Result<WggProduct> {
    let mut result = WggProduct {
        id: product.id,
        name: product.name,
        description: {
            // Our model assumes there is always a description, so we'll just make an empty one if it doesn't exist.
            let desc = product.description.unwrap_or_default();
            let out = if let Some(extra) = desc.extension {
                desc.main + &extra
            } else {
                desc.main
            };

            Description {
                text: out,
                text_type: TextType::Markdown,
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
            .map(|url| wgg_picnic::images::image_url(url.image_id, ImageSize::Medium))
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
fn parse_picnic_item_to_search_item(article: wgg_picnic::models::SingleArticle) -> WggSearchProduct {
    // Note that Picnic's 'display_price' is equivalent to our 'full_price'.
    let mut result = WggSearchProduct {
        id: article.id,
        name: article.name,
        full_price: article.display_price,
        display_price: article.display_price,
        unit_quantity: Default::default(),
        unit_price: None,
        available: true,
        image_url: Some(wgg_picnic::images::image_url(article.image_id, ImageSize::Medium)),
        decorators: Vec::new(),
        provider: Provider::Picnic,
    };

    // Parse remaining decorators
    for dec in article.decorators {
        parse_decorator(
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
                replacements: replacements.into_iter().map(parse_picnic_item_to_search_item).collect(),
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
                    .map(|id| wgg_picnic::images::image_url(id, ImageSize::Medium))
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