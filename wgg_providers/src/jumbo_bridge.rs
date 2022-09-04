use crate::common_bridge::{derive_unit_price, parse_unit_component};
use crate::models::{Decorator, SaleLabel, SaleValidity, UnavailableItem, UnavailableReason, UnitPrice, UnitQuantity};
use crate::Result;
use crate::{common_bridge, Autocomplete, OffsetPagination, Provider, ProviderInfo, SearchItem};
use cached::proc_macro::once;
use wgg_jumbo::models::AvailabilityType;
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
    fn provider() -> Provider {
        Provider::Jumbo
    }

    #[tracing::instrument(name = "jumbo_autocomplete", level="debug", skip_all, fields(query = query))]
    async fn autocomplete(&self, query: &str) -> crate::Result<Vec<Autocomplete>> {
        // Cache the response for a day at a time, as the Jumbo autocomplete is just a giant list of terms.
        #[once(time = 86400, result = true)]
        async fn inner(api: &BaseJumboApi) -> Result<Vec<Autocomplete>> {
            Ok(api
                .autocomplete()
                .await?
                .autocomplete
                .data
                .into_iter()
                .map(|auto| Autocomplete { name: auto })
                .collect())
        }

        let response = inner(&self.api).await?;
        // TODO: Better matching
        Ok(response.into_iter().filter(|c| c.name.contains(query)).collect())
    }

    #[tracing::instrument(name = "jumbo_search", level = "debug", skip(self))]
    async fn search(&self, query: &str, offset: Option<u32>) -> Result<OffsetPagination<SearchItem>> {
        let search_results = self.api.search(query, offset, None).await?;

        Ok(OffsetPagination {
            items: search_results
                .products
                .data
                .into_iter()
                .map(|i| parse_jumbo_item_to_search_item(&self.api, i))
                .collect(),
            total_items: search_results.products.total as usize,
            offset: search_results.products.offset,
        })
    }
}

/// Parse a full picnic [wgg_jumbo::models::SingleArticle] to our normalised [SearchItem]
fn parse_jumbo_item_to_search_item(jumbo_api: &BaseJumboApi, article: wgg_jumbo::models::PartialProduct) -> SearchItem {
    let mut result = SearchItem {
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
                .map(Decorator::SaleLabel),
        );
        result.decorators.push(Decorator::SaleValidity(SaleValidity {
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

        result.decorators.push(Decorator::Unavailable(unavailable));
    }

    result
}
