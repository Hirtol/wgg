use crate::common_bridge::parse_quantity;
use crate::models::{CentPrice, FreshLabel, SaleLabel, SaleValidity, UnavailableItem, UnitPrice};
use crate::{common_bridge, Autocomplete, OffsetPagination, Provider, ProviderInfo, SearchItem};
use chrono::{Datelike, LocalResult, NaiveDate, TimeZone};
use wgg_picnic::models::{Decorator, UnavailableReason};
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

    #[tracing::instrument(name = "picnic_autocomplete", level = "debug", skip(self))]
    async fn autocomplete(&self, query: &str) -> Result<Vec<Autocomplete>> {
        let result = self.api.suggestions(query).await?;

        Ok(result
            .into_iter()
            .map(|i| Autocomplete { name: i.suggestion })
            .collect())
    }

    #[tracing::instrument(name = "picnic_autocomplete", level = "debug", skip(self, _offset))]
    async fn search(&self, query: &str, _offset: Option<u32>) -> Result<OffsetPagination<SearchItem>> {
        let result = self.api.search(query).await?;

        let result: Vec<SearchItem> = result
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
}

/// Parse a full picnic [wgg_picnic::models::SingleArticle] to our normalised [SearchItem]
fn parse_picnic_item_to_search_item(picnic_api: &PicnicApi, article: wgg_picnic::models::SingleArticle) -> SearchItem {
    // Note that Picnic's 'display_price' is equivalent to our 'full_price'.
    let mut result = SearchItem {
        id: article.id,
        name: article.name,
        full_price: article.display_price,
        display_price: article.display_price,
        unit_quantity: parse_quantity(&article.unit_quantity).unwrap_or_default(),
        unit_price: None,
        available: true,
        image_url: Some(
            picnic_api
                .image_url(article.image_id, wgg_picnic::models::ImageSize::Large)
                .to_string(),
        ),
        decorators: Vec::new(),
        provider: Provider::Picnic,
    };

    // Parse remaining decorators
    for dec in article.decorators {
        match dec {
            Decorator::FreshLabel { period } => {
                if let Some(days_fresh) = parse_days_fresh(&period) {
                    let fresh_label = FreshLabel { days_fresh };

                    result
                        .decorators
                        .push(crate::models::Decorator::FreshLabel(fresh_label))
                }
            }
            Decorator::Label { text } => {
                let sale_label = SaleLabel { text };

                result.decorators.push(crate::models::Decorator::SaleLabel(sale_label))
            }
            Decorator::Price { display_price } => {
                // Decorator price is the price *including* current sales if available.
                result.display_price = display_price
            }
            Decorator::ValidityLabel { valid_until } => {
                if let LocalResult::Single(valid_until) =
                    chrono::Utc.from_local_datetime(&valid_until.and_hms(23, 59, 59))
                {
                    let valid_from =
                        NaiveDate::from_isoywd(valid_until.year(), valid_until.iso_week().week(), chrono::Weekday::Mon)
                            .and_hms(0, 0, 0);
                    let valid_from = if let Some(time) = chrono::Utc.from_local_datetime(&valid_from).single() {
                        time
                    } else {
                        continue;
                    };
                    let sale_validity = SaleValidity {
                        valid_from,
                        valid_until,
                    };

                    result
                        .decorators
                        .push(crate::models::Decorator::SaleValidity(sale_validity))
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

                result.available = false;
                result
                    .decorators
                    .push(crate::models::Decorator::Unavailable(unavailable))
            }
            _ => {}
        }
    }

    // Parse unit quantity
    if let Some(unit_price_str) = &article.unit_quantity_sub {
        result.unit_price = parse_unit_price(unit_price_str)
            .or_else(|| common_bridge::derive_unit_price(&result.unit_quantity, result.display_price));
    } else {
        result.unit_price = common_bridge::derive_unit_price(&result.unit_quantity, result.display_price)
    }

    result
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
    use crate::common_bridge::{derive_unit_price, parse_quantity};
    use crate::models::{Unit, UnitPrice};
    use crate::picnic_bridge::{parse_days_fresh, parse_euro_price, parse_unit_price};

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
    pub fn test_derive_unit_price() {
        let unit_prices = vec![("250 gram", 242), ("10 stuks M/L", 379), ("1.5 liter", 150)];
        let expected = vec![
            UnitPrice {
                unit: Unit::KiloGram,
                price: 968,
            },
            UnitPrice {
                unit: Unit::Piece,
                price: 38,
            },
            UnitPrice {
                unit: Unit::Liter,
                price: 100,
            },
        ];

        assert_eq!(
            unit_prices
                .into_iter()
                .flat_map(|(quantity, price)| derive_unit_price(&parse_quantity(quantity).unwrap_or_default(), price))
                .collect::<Vec<_>>(),
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
}
