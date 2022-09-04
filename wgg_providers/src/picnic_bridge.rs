use crate::models::{CentPrice, FreshLabel, SaleLabel, SaleValidity, UnavailableItem, Unit, UnitPrice};
use crate::{Autocomplete, Provider, ProviderInfo, SearchItem};
use chrono::{LocalResult, TimeZone};
use wgg_picnic::models::{Decorator, UnavailableReason};
use wgg_picnic::PicnicApi;

use crate::Result;

#[async_trait::async_trait]
impl ProviderInfo for PicnicApi {
    async fn autocomplete(&self, query: &str) -> Result<Vec<Autocomplete>> {
        let result = self.suggestions(query).await?;

        Ok(result
            .into_iter()
            .map(|i| Autocomplete { name: i.suggestion })
            .collect())
    }

    async fn search(&self, query: &str) -> Result<Vec<SearchItem>> {
        let result = self.search(query).await?;

        let result = result
            .into_iter()
            .flat_map(|res| {
                res.items.into_iter().filter_map(|item| {
                    if let wgg_picnic::models::SearchItem::SingleArticle(article) = item {
                        Some(parse_picnic_item_to_search_item(self, article))
                    } else {
                        None
                    }
                })
            })
            .collect();

        Ok(result)
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
        unit_quantity: article.unit_quantity,
        unit_price: None,
        unavailable: false,
        image_url: picnic_api
            .image_url(article.image_id, wgg_picnic::models::ImageSize::Large)
            .to_string(),
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
                    let sale_validity = SaleValidity { valid_until };

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

                result.unavailable = true;
                result
                    .decorators
                    .push(crate::models::Decorator::Unavailable(unavailable))
            }
            _ => {}
        }
    }

    // Parse unit quantity
    if let Some(unit_price_str) = &article.unit_quantity_sub {
        result.unit_price =
            parse_unit_price(unit_price_str).or_else(|| derive_unit_price(&result.unit_quantity, result.display_price));
    } else {
        result.unit_price = derive_unit_price(&result.unit_quantity, result.display_price)
    }

    result
}

/// Try to derive a unit price from the unit quantity and display price.
///
/// Preferably one would first use [parse_unit_price], but this function is available as a fallback.
fn derive_unit_price(unit_quantity: &str, display_price: CentPrice) -> Option<UnitPrice> {
    let mut components = unit_quantity.split_whitespace();
    let quantity: f64 = components.next()?.parse().ok()?;
    let unit = components.next()?;

    let (normalised_quantity, normalised_unit) = match unit {
        "gram" => ((quantity / 1000.), Unit::Kg),
        _ => (quantity, parse_unit_component(unit)?),
    };

    UnitPrice {
        unit: normalised_unit,
        price: (display_price as f64 / normalised_quantity).round() as CentPrice,
    }
    .into()
}

/// Try to parse the provided unit price in the format `€16.54/l` or `€13.54/kg`.
///
/// Invalid input will return [None]
fn parse_unit_price(unit_price: &str) -> Option<UnitPrice> {
    if let Some((price_component, unit_component)) = unit_price.split_once('/') {
        let cent_price = parse_euro_price(price_component);
        let unit = parse_unit_component(unit_component);

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

/// Try to parse the provided price in the format `l` or `kg` as a [crate::models::Unit].
///
/// Invalid input will return [None]
fn parse_unit_component(unit: &str) -> Option<Unit> {
    match unit {
        "l" => Some(Unit::Liter),
        "liter" => Some(Unit::Liter),
        "kg" => Some(Unit::Kg),
        "stuk" => Some(Unit::Piece),
        "stuks" => Some(Unit::Piece),
        "piece" => Some(Unit::Piece),
        _ => None,
    }
}

/// Parse the days fresh in the format `P5D` or `P7D` (5 days and 7 days respectively).
fn parse_days_fresh(period: &str) -> Option<u32> {
    let price: String = period.chars().filter(|&ch| ch != 'P' && ch != 'D').collect();

    price.parse().ok()
}

#[cfg(test)]
mod test {
    use crate::models::{Unit, UnitPrice};
    use crate::picnic_bridge::{derive_unit_price, parse_days_fresh, parse_euro_price, parse_unit_price};

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
                unit: Unit::Kg,
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
                unit: Unit::Kg,
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
                .flat_map(|(quantity, price)| derive_unit_price(quantity, price))
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
