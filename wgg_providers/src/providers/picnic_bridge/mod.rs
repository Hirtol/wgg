use std::marker::PhantomData;
use std::num::NonZeroU32;
use std::ops::Deref;
use std::time::Duration;

use chrono::{Datelike, NaiveDate, Utc};
use futures::future::FutureExt;
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Jitter, Quota};
use itertools::Itertools;
use regex::Regex;
use secrecy::ExposeSecret;

pub use authentication::PicnicCredentials;
use wgg_picnic::models::{
    Body, Decorator, ImageSize, PageBody, PageChildren, PagePml, PagesRoot, PmlComponent, UnavailableReason,
};
use wgg_picnic::PicnicApi;

use crate::error::Result;
use crate::models::{
    AllergyTags, AllergyType, CentPrice, Description, FreshLabel, IngredientInfo, ItemInfo, ItemType, NutritionalInfo,
    NutritionalItem, PrepTime, PriceInfo, Provider, ProviderMetadata, SaleInformation, SaleResolutionStrategy,
    SaleValidity, SubNutritionalItem, TextType, UnavailableItem, UnitPrice, WggAutocomplete, WggDecorator, WggProduct,
    WggSaleCategory, WggSaleGroupComplete, WggSaleGroupLimited, WggSaleItem, WggSearchProduct,
};
use crate::pagination::OffsetPagination;
use crate::providers::common_bridge::{parse_quantity, parse_sale_label};
use crate::providers::{common_bridge, ProviderInfo, StaticProviderInfo};
use crate::{lazy_re, lazy_re_set, ProviderError};

mod authentication;

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

impl StaticProviderInfo for PicnicBridge {
    fn provider() -> Provider {
        Provider::Picnic
    }

    fn metadata() -> ProviderMetadata {
        ProviderMetadata {
            logo_url: "https://upload.wikimedia.org/wikipedia/commons/0/01/Picnic_logo.svg".into(),
            // Technically Picnic sale resolution depends on the order of addition to the cart, but the user can manually
            // make this opportunistic in that case.
            sale_strategy: SaleResolutionStrategy::Opportunistic,
        }
    }
}

impl PicnicBridge {
    pub(crate) async fn new(credentials: PicnicCredentials, limit_rps: Option<NonZeroU32>) -> Result<Self> {
        let config = Default::default();
        let api = if let Some(auth_token) = credentials.to_credentials() {
            PicnicApi::new(auth_token, config)
        } else {
            let result =
                PicnicApi::from_login(credentials.email(), credentials.password().expose_secret(), config).await?;

            tracing::info!("Picnic remote login complete");

            result
        };

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

    pub(crate) async fn credentials(&self) -> wgg_picnic::Credentials {
        self.api.read().await.credentials().clone()
    }

    async fn wait_rate_limit(&self) {
        // Only use Jitter if we have to start delaying calls.
        if self.limiter.check().is_err() {
            self.limiter.until_ready_with_jitter(Jitter::up_to(JITTER)).await
        }
    }

    async fn picnic_request<'a, F, O>(&self, api_request: F) -> Result<O>
    where
        for<'b> F: Fn(&'b TemporaryApi<'b, 'a>) -> futures::future::BoxFuture<'b, wgg_picnic::Result<O>>,
    {
        self.wait_rate_limit().await;

        let api_result = {
            let read_lock = self.api.read().await;
            api_request(&TemporaryApi::new(&read_lock)).await
        };

        let result = match api_result {
            Ok(res) => res,
            Err(wgg_picnic::ApiError::AuthError) => {
                if let Ok(_lock) = self.refresh_lock.try_lock() {
                    tracing::info!("Refreshing Picnic authentication token due to failure");
                    let mut api_guard = self.api.write().await;

                    api_guard
                        .login(self.credentials.email(), self.credentials.password().expose_secret())
                        .await?;

                    api_request(&TemporaryApi::new(&api_guard)).await?
                } else {
                    // Once the refresh_lock is released we will *eventually* be able to lock it here and re-execute the request.
                    // If it fails again just error out.
                    let _ = self.refresh_lock.lock().await;

                    let read_lock = self.api.read().await;
                    api_request(&TemporaryApi::new(&read_lock)).await?
                }
            }
            Err(e) => return Err(e.into()),
        };

        Ok(result)
    }
}

// These implementations contain a lot of duplicate code to handle the Auth token refresh (when needed).
// Unfortunately, extracting these out to a common function taking a closure required higher-lifetime bounds I couldn't get to work (even with boxing :/)
#[async_trait::async_trait]
impl ProviderInfo for PicnicBridge {
    fn provider(&self) -> Provider {
        <Self as StaticProviderInfo>::provider()
    }

    fn metadata(&self) -> ProviderMetadata {
        <Self as StaticProviderInfo>::metadata()
    }

    #[tracing::instrument(name = "picnic_autocomplete", level = "trace", skip(self))]
    async fn autocomplete(&self, query: &str) -> Result<Vec<WggAutocomplete>> {
        let result = self.picnic_request(|api| api.suggestions(query).boxed()).await?;

        #[cfg(feature = "trace-original-api")]
        tracing::trace!("Picnic Autocomplete: {:#?}", result);

        Ok(result
            .into_iter()
            .map(|i| WggAutocomplete { name: i.suggestion })
            .collect())
    }

    #[tracing::instrument(name = "picnic_search", level = "trace", skip(self))]
    async fn search(&self, query: &str, offset: Option<u32>) -> Result<OffsetPagination<WggSearchProduct>> {
        let result = self.picnic_request(|api| api.search(query).boxed()).await?;
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

    #[tracing::instrument(name = "picnic_product", level = "trace", skip(self))]
    async fn product(&self, product_id: &str) -> Result<WggProduct> {
        let result = self.picnic_request(|api| api.product(product_id).boxed()).await?;

        #[cfg(feature = "trace-original-api")]
        tracing::trace!("Picnic Product: {:#?}", result);

        parse_picnic_full_product_to_product(result)
    }

    #[tracing::instrument(name = "picnic_promotions", level = "trace", skip(self))]
    async fn promotions(&self) -> Result<Vec<WggSaleCategory>> {
        let result = self.picnic_request(|api| api.promotions().boxed()).await?;

        #[cfg(feature = "trace-original-api")]
        tracing::trace!("Picnic Promotions: {:#?}", result);

        Ok(parse_new_picnic_promotions(result))
    }

    #[tracing::instrument(name = "picnic_promotions_sublist", level = "trace", skip(self))]
    async fn promotions_sublist(&self, sublist_id: &str) -> Result<WggSaleGroupComplete> {
        // When querying for a sublist with `depth > 1` we just get a raw array of SingleArticles
        let result = self.picnic_request(|api| api.promotion(sublist_id).boxed()).await?;

        #[cfg(feature = "trace-original-api")]
        tracing::trace!("Picnic Promotions Sublist: {:#?}", result);

        let result = parse_new_picnic_promotion(sublist_id, result).ok_or_else(|| ProviderError::NothingFound)?;

        Ok(result)
    }
}

// This is here to hack around the lack of bounding for higher-ranked lifetimes
// With thanks to: https://users.rust-lang.org/t/argument-requires-that-is-borrowed-for-static/66503/2
struct TemporaryApi<'lower, 'upper: 'lower> {
    api: &'lower PicnicApi,
    _upper: PhantomData<&'upper ()>,
}

impl<'lower, 'upper> Deref for TemporaryApi<'lower, 'upper> {
    type Target = PicnicApi;

    fn deref(&self) -> &Self::Target {
        self.api
    }
}

impl<'lower, 'upper> TemporaryApi<'lower, 'upper> {
    pub fn new(api: &'lower PicnicApi) -> Self {
        Self {
            api,
            _upper: Default::default(),
        }
    }
}

fn parse_new_picnic_promotions(promotions: PagesRoot) -> Vec<WggSaleCategory> {
    //language=regexp
    lazy_re_set!(
        BLOCK_IDS,
        // Normal group as was the case in the old Picnic client
        r#"promo-groups-vertical-tiles.*"#,
        // Vertical list of single item groups
        r#"promo-group-list-element-section.*"#
    );

    let children = promotions.body.children;
    // Used for some 'list-tiles'. They don't _always_ have titles.
    let mut misc_list_id = 0;

    children
        .into_iter()
        .flat_map(|groups| groups.to_block())
        .flat_map(|promo_group| {
            let match_idx = BLOCK_IDS.0.matches(&promo_group.id).into_iter().next()?;
            match match_idx {
                0 => parse_promo_vertical_tiles(promo_group),
                1 => parse_promo_list_tiles(promo_group, &mut misc_list_id),
                _ => unreachable!(),
            }
        })
        .collect()
}

fn parse_promo_vertical_tiles(promotion: PageBody) -> Option<WggSaleCategory> {
    //language=regexp
    lazy_re!(PROMO_GROUP_ID, r".*promo-group-(.*)");

    let child = promotion.children.into_iter().next()?.to_block()?;
    // Expect some form of `promo-groups-vertical-tiles`
    if child.id.contains("inner") {
        let nested_child = child.children.into_iter().last()?.to_block()?;
        // Now we can expect some form of id: `single-promo-group-b50f7476-2ade-4af8-888a-ac97bfe8b539`
        let promo_id = PROMO_GROUP_ID.captures(&nested_child.id)?.get(1)?.as_str();
        // The first item is the title in PML format, we can get it from ArticleAnalytics instead.
        let article_children = nested_child.children.into_iter().nth(1)?;
        let article_block = article_children.to_block()?;
        // Whether to display the `more` button in the categories
        let more_button = matches!(article_block.children.last(), Some(PageChildren::Pml(_)));

        let mut category_name = None;
        let articles = article_block
            .children
            .into_iter()
            .flat_map(PageChildren::to_article_tile)
            .map(|article| {
                let parsed = parse_picnic_item_to_search_item(article.article);

                // Update the `category_name`.
                if category_name.is_none() {
                    if let Some(category) = article.analytics.contexts.into_iter().last() {
                        category_name = category.data.name;
                    }
                }

                WggSaleItem::Product(parsed)
            })
            .collect();

        Some(WggSaleCategory {
            id: Some(promo_id.to_string()),
            name: category_name.unwrap_or_else(|| "UNKNOWN".to_string()),
            items: articles,
            image_urls: vec![],
            complete: !more_button,
            provider: Provider::Picnic,
        })
    } else {
        tracing::warn!(
            block_id = child.id,
            "Possibly outdated Picnic promotion parsing, expected `inner` id"
        );
        None
    }
}

fn parse_promo_list_tiles(promotion: PageBody, misc_list_id: &mut u32) -> Option<WggSaleCategory> {
    // Check if it's a 'naked' list tile section, without header
    let child = promotion.children.into_iter().last()?.to_block()?;
    // Expect some form of `promo-groups-vertical-tiles`
    if child.id.contains("section-content") {
        let pml_articles = child.children.into_iter().flat_map(PageChildren::to_pml);

        let mut category_name = None;
        let sale_groups = pml_articles
            .flat_map(|article| parse_list_article_item(&mut category_name, article))
            .collect();

        Some(WggSaleCategory {
            id: None,
            name: category_name.unwrap_or_else(|| {
                *misc_list_id += 1;
                format!("Misc List {misc_list_id}")
            }),
            items: sale_groups,
            image_urls: vec![],
            complete: true,
            provider: Provider::Picnic,
        })
    } else {
        tracing::warn!(
            block_id = child.id,
            "Possibly outdated Picnic promotion parsing, expected `inner` id"
        );
        None
    }
}

fn parse_list_article_item(category_name: &mut Option<String>, article: PagePml) -> Option<WggSaleItem> {
    //language=regexp
    lazy_re!(PROMO_GROUP_ID, r".*promo_group_id=(.*)");

    // Usually 3 items in `analytics`. First is skipped, second is the group id, third is the category title.
    let mut category_and_id = article.analytics.contexts.into_iter().skip(1);
    let binding = category_and_id.next()?.data.deeplink?;

    let group_id = PROMO_GROUP_ID.captures(&binding)?.get(1)?.as_str();
    // Update the `category_name`.
    if category_name.is_none() {
        if let Some(category) = category_and_id.last() {
            *category_name = category.data.name;
        }
    }

    let touchable_outline = article.pml.component?.to_touchable()?;
    let sale_accessibility = touchable_outline.accessibility_label?;
    let child = touchable_outline.child?.to_container()?.child?.to_stack()?;

    let mut stack_children = child.children.into_iter();
    let image_id = stack_children.next()?.to_container()?.child?.to_image()?.source.id;

    let article_data = stack_children.next()?.to_container()?.child?.to_stack()?.children;
    let mut article_iter = article_data.into_iter();

    let mut title_items = article_iter.next()?.to_stack()?.children.into_iter();
    let binding_title = title_items.next()?.to_container()?.child?.to_rich_text()?.markdown;
    let title = parse_pml_markdown(&binding_title)?;
    let description =
        parse_pml_markdown(&title_items.next()?.to_container()?.child?.to_rich_text()?.markdown).map(|c| c.to_string());

    // The accessibility label is made up of `PRODUCT TITLE + SALE STRING`, we just care about the latter.
    let sale_type = sale_accessibility.replace(title, "");
    let parsed_sale = common_bridge::parse_sale_label(&sale_type);

    let sale_information = {
        let valid = common_bridge::get_guessed_sale_validity(Utc::now());
        if let Some(sale) = parsed_sale {
            SaleInformation {
                label: sale_type,
                additional_label: Vec::new(),
                sale_validity: valid,
                sale_type: Some(sale),
            }
        } else {
            SaleInformation {
                label: "UNKNOWN".to_string(),
                additional_label: Vec::new(),
                sale_validity: valid,
                sale_type: None,
            }
        }
    };

    let group = WggSaleGroupLimited {
        id: group_id.to_string(),
        name: title.to_string(),
        image_urls: vec![wgg_picnic::images::image_url(image_id, ImageSize::Small)],
        items: Vec::new(),
        sale_info: sale_information,
        sale_description: description,
        provider: Provider::Picnic,
    };

    Some(WggSaleItem::Group(group))
}

fn parse_new_picnic_promotion(sublist_id: impl Into<String>, promotion: PagesRoot) -> Option<WggSaleGroupComplete> {
    let children = promotion.body.children;
    let content = children.into_iter().next()?.to_block()?;
    let vertical_tiles = content.children.into_iter().next()?.to_block()?;

    if vertical_tiles.id.contains("vertical-selling-unit-tiles") {
        let articles: Vec<_> = vertical_tiles
            .children
            .into_iter()
            .flat_map(PageChildren::to_article_tile)
            .map(|article| parse_picnic_item_to_search_item(article.article))
            .collect();

        let sale_info: SaleInformation = articles
            .iter()
            .flat_map(|item| &item.sale_information)
            .next()
            .cloned()
            .unwrap_or_else(|| SaleInformation {
                label: "UNKNOWN".to_string(),
                additional_label: Vec::new(),
                sale_validity: common_bridge::get_guessed_sale_validity(Utc::now()),
                sale_type: None,
            });

        let result = WggSaleGroupComplete {
            id: sublist_id.into(),
            name: promotion.header.title,
            image_urls: vec![],
            items: articles,
            sale_info,
            sale_description: None,
            provider: Provider::Picnic,
        };

        Some(result)
    } else {
        tracing::warn!(
            ?vertical_tiles,
            "Expected `vertical-selling-unit-tiles` id, but couldn't match with given id"
        );
        None
    }
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
        unavailable_details: None,
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
        additional_items: Vec::new(),
        // Will be parsed
        decorators: Vec::new(),
        // Will be parsed
        sale_information: None,
        provider: Provider::Picnic,
    };

    // Parse unit quantity
    if let Some(quantity) = parse_quantity(&product.unit_quantity) {
        result.unit_quantity = quantity;
    } else {
        // Since we couldn't parse a 'normal' quantity it might be of an unconventional form such as:
        // `4-6 pers | 30 mins`, we can extract the prep time!
        if let Some(minutes) = parse_prep_time(&product.unit_quantity) {
            result
                .decorators
                .push(WggDecorator::PrepTime(PrepTime { time_minutes: minutes }));
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
        if let Body::Pml { pml_content } = &blob.body {
            if let Some(PmlComponent::RichText(item)) = &pml_content.component {
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
        lazy_re!(REGEX, r"(\d+) (dagen|dag|week|weken)");

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

    // Parse Sales
    let (_, sale_validity) = parse_decorators_for_sale(&product.decorators);
    if let Some(promo) = product.labels.promo {
        result.sale_information = SaleInformation {
            sale_type: parse_sale_label(&promo.text),
            label: promo.text,
            additional_label: Vec::new(),
            sale_validity: sale_validity.unwrap_or_else(|| common_bridge::get_guessed_sale_validity(Utc::now())),
        }
        .into();
    }

    // Parse remaining decorators
    for dec in product.decorators {
        parse_decorator(
            dec,
            &mut result.decorators,
            &mut result.price_info.display_price,
            &mut result.unavailable_details,
        )
    }

    // Parse misc items
    for item in product.misc {
        // We already parse ingredients/nutritional info earlier.
        if item.header.text == "Ingrediënten" || item.header.text == "Voedingswaarde" {
            continue;
        }
        match item.body {
            Body::Pml { pml_content } => match pml_content.component {
                Some(PmlComponent::Stack(stack)) => {
                    for child in stack.children.into_iter() {
                        let md = child.to_rich_text().map(|md| md.markdown);
                        if let Some(md) = md {
                            if md.contains("Bewaren") {
                                result.additional_items.push(ItemInfo {
                                    item_type: ItemType::StorageAdvice,
                                    text: md,
                                    text_type: TextType::Markdown,
                                })
                            } else if md.contains("Land van herkomst") {
                                result.additional_items.push(ItemInfo {
                                    item_type: ItemType::CountryOfOrigin,
                                    text: md,
                                    text_type: TextType::Markdown,
                                })
                            }
                        }
                    }
                }
                Some(PmlComponent::RichText(text)) => {
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
                _ => {
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
        unit_quantity: Default::default(),
        image_url: Some(wgg_picnic::images::image_url(article.image_id, ImageSize::Medium)),
        decorators: Vec::new(),
        sale_information: None,
        provider: Provider::Picnic,
        price_info: PriceInfo {
            display_price: article.display_price,
            original_price: article.display_price,
            unit_price: None,
        },
        unavailable_details: None,
    };

    // Parse sale data
    let (sale_label, sale_validity) = parse_decorators_for_sale(&article.decorators);
    if let Some(label) = sale_label {
        if label != "Receptkorting" {
            result.sale_information = SaleInformation {
                sale_type: parse_sale_label(&label),
                label,
                additional_label: Vec::new(),
                sale_validity: sale_validity.unwrap_or_else(|| common_bridge::get_guessed_sale_validity(Utc::now())),
            }
            .into()
        }
    }

    // Parse remaining decorators
    for dec in article.decorators {
        parse_decorator(
            dec,
            &mut result.decorators,
            &mut result.price_info.display_price,
            &mut result.unavailable_details,
        )
    }

    // Parse unit quantity
    if let Some(quantity) = parse_quantity(&article.unit_quantity) {
        result.unit_quantity = quantity;
    } else {
        // Since we couldn't parse a 'normal' quantity it might be of an unconventional form such as:
        // `4-6 pers | 30 mins`, we can extract the prep time!
        if let Some(minutes) = parse_prep_time(&article.unit_quantity) {
            result
                .decorators
                .push(WggDecorator::PrepTime(PrepTime { time_minutes: minutes }));
        }
    }

    // Parse unit price quantity
    if let Some(unit_price_str) = &article.unit_quantity_sub {
        result.price_info.unit_price = parse_unit_price(unit_price_str)
            .or_else(|| common_bridge::derive_unit_price(&result.unit_quantity, result.price_info.display_price));
    } else {
        result.price_info.unit_price =
            common_bridge::derive_unit_price(&result.unit_quantity, result.price_info.display_price)
    }

    result
}

fn parse_decorators_for_sale(decorators: &[Decorator]) -> (Option<String>, Option<SaleValidity>) {
    let mut sale_label = None;
    let mut sale_validity = None;

    for dec in decorators {
        match dec {
            Decorator::Promo { text } => {
                sale_label = Some(text.clone());
            }
            Decorator::ValidityLabel { valid_until } => {
                let valid_until =
                    chrono::DateTime::from_utc(valid_until.and_hms_opt(23, 59, 59).expect("invalid time"), Utc);
                let valid_from =
                    NaiveDate::from_isoywd_opt(valid_until.year(), valid_until.iso_week().week(), chrono::Weekday::Mon)
                        .expect("invalid time")
                        .and_hms_opt(0, 0, 0)
                        .expect("invalid time");
                let valid_from = chrono::DateTime::from_utc(valid_from, Utc);

                sale_validity = Some(SaleValidity {
                    valid_from,
                    valid_until,
                });
            }
            _ => {}
        }
    }

    (sale_label, sale_validity)
}

// Encourage inlining to get rid of the Option costs.
#[inline(always)]
pub fn parse_decorator(
    decorator: Decorator,
    result: &mut Vec<WggDecorator>,
    set_display_price: &mut u32,
    set_unavailable: &mut Option<UnavailableItem>,
) {
    match decorator {
        // If we already parsed it above, we don't want to do it again!
        Decorator::FreshLabel { period } if !result.iter().any(|i| matches!(i, WggDecorator::FreshLabel(_))) => {
            if let Some(days_fresh) = parse_days_fresh(&period) {
                let fresh_label = FreshLabel { days_fresh };

                result.push(WggDecorator::FreshLabel(fresh_label))
            }
        }
        Decorator::Price { display_price } => {
            // Decorator price is the price *including* current sales if available.
            *set_display_price = display_price
        }
        Decorator::Unavailable {
            reason,
            replacements,
            explanation,
        } => {
            *set_unavailable = UnavailableItem {
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
            }
            .into();
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

/// Parse any PML markdown between their colour tags.
fn parse_pml_markdown(contents: &str) -> Option<&str> {
    //language=regexp
    lazy_re!(ESCAPER, r"#.*?\)(.*?)#.*");

    let matchr = ESCAPER.captures(contents)?.get(1)?.as_str();

    Some(matchr)
}

#[cfg(test)]
mod test {
    use std::vec;

    use crate::models::{IngredientInfo, Unit, UnitPrice};
    use crate::providers::picnic_bridge::{
        parse_days_fresh, parse_euro_price, parse_picnic_ingredient_blob, parse_prep_time, parse_unit_price,
    };

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
