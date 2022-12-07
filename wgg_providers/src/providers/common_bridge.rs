use crate::models::{CentPrice, SaleValidity, Unit, UnitPrice, UnitQuantity, WggDecorator};
use chrono::{DateTime, Datelike, Utc, Weekday};
use std::borrow::Cow;

/// Try to parse the provided price in the format `l` or `kg` as a [crate::models::Unit].
///
/// Invalid input will return [None]
pub(crate) fn parse_unit_component(unit: &str) -> Option<Unit> {
    match unit {
        "l" => Some(Unit::Liter),
        "L" => Some(Unit::Liter),
        "liter" => Some(Unit::Liter),
        "ml" => Some(Unit::MilliLiter),
        "kg" => Some(Unit::KiloGram),
        "Kg" => Some(Unit::KiloGram),
        "KG" => Some(Unit::KiloGram),
        "g" => Some(Unit::Gram),
        "G" => Some(Unit::Gram),
        "gram" => Some(Unit::Gram),
        "stuk" => Some(Unit::Piece),
        "stuks" => Some(Unit::Piece),
        "piece" => Some(Unit::Piece),
        "pieces" => Some(Unit::Piece),
        _ => None,
    }
}

/// Parse a [UnitQuantity] of the form `500 g` or `10 x 55 g` or `1 liter`.
pub(crate) fn parse_quantity(quantity: &str) -> Option<UnitQuantity> {
    // If it's in the format `10 x 55 g`
    if quantity.contains('x') {
        let mut whitespaces = quantity.split_whitespace();
        let multiplier = whitespaces.next()?;
        let _ = whitespaces.next()?;

        let (quantity, unit) = parse_quantity_unit(whitespaces)?;
        let multiplier: f64 = multiplier.parse().ok()?;

        UnitQuantity {
            unit,
            amount: (quantity * multiplier).round(),
        }
        .into()
    } else {
        let (quantity, unit) = parse_quantity_unit(quantity.split_whitespace())?;

        UnitQuantity { unit, amount: quantity }.into()
    }
}

/// Attempt to parse a `(Quantity, Unit)` combination adhering to either of the following formats:
/// * `55 g`
/// * `500ml`
fn parse_quantity_unit<'a>(mut whitespaces: impl Iterator<Item = &'a str>) -> Option<(f64, Unit)> {
    let quantity = whitespaces.next()?.replace(',', ".");
    // If we can parse in one go we know that we're dealing with a format of `55 g`.
    if let Ok(quant) = quantity.parse::<f64>() {
        let unit = parse_unit_component(whitespaces.next()?)?;
        Some((quant, unit))
    } else {
        // Otherwise we'll need to try and split them.
        let (index, _) = quantity
            .char_indices()
            .take_while(|(_, chr)| chr.is_ascii_digit())
            .last()?;

        let (quantity_part, unit_part) = quantity.split_at(index + 1);

        Some((quantity_part.parse().ok()?, parse_unit_component(unit_part)?))
    }
}

/// Try to derive a unit price from the unit quantity and display price.
///
/// Preferably one would first use [parse_unit_price], but this function is available as a fallback.
pub(crate) fn derive_unit_price(unit_quantity: &UnitQuantity, display_price: CentPrice) -> Option<UnitPrice> {
    let (normalised_quantity, normalised_unit) = match unit_quantity.unit {
        Unit::Gram => ((unit_quantity.amount / 1000.), Unit::KiloGram),
        Unit::MilliLiter => ((unit_quantity.amount / 1000.), Unit::Liter),
        _ => (unit_quantity.amount, unit_quantity.unit),
    };

    UnitPrice {
        unit: normalised_unit,
        price: (display_price as f64 / normalised_quantity).round() as CentPrice,
    }
    .into()
}

/// Get either a [SaleValidity] from the given decorators, or make a guess based on the current time.
///
/// See [get_guessed_sale_validity] for more.
pub(crate) fn get_sale_validity<'a>(decorators: impl IntoIterator<Item = &'a WggDecorator>) -> Cow<'a, SaleValidity> {
    decorators
        .into_iter()
        .flat_map(|i| match &i {
            WggDecorator::SaleValidity(valid) => Some(Cow::Borrowed(valid)),
            _ => None,
        })
        .next()
        .unwrap_or_else(|| Cow::Owned(get_guessed_sale_validity(Utc::now())))
}

/// Return a best estimate sale validity date.
/// Assumes that the sale started at the beginning of `now`'s week (Monday 00:00:00) and will last until the end of `now`'s
/// week (Sunday 23:59:59).
pub(crate) fn get_guessed_sale_validity(now: DateTime<Utc>) -> SaleValidity {
    // We assume a sale is valid until the very end of this week
    let monday =
        chrono::NaiveDate::from_isoywd(now.iso_week().year(), now.iso_week().week(), Weekday::Mon).and_hms(0, 0, 0);
    let sunday =
        chrono::NaiveDate::from_isoywd(now.iso_week().year(), now.iso_week().week(), Weekday::Sun).and_hms(23, 59, 59);

    let valid_from: DateTime<Utc> = DateTime::from_local(monday, Utc);
    let valid_until: DateTime<Utc> = DateTime::from_local(sunday, Utc);

    SaleValidity {
        valid_from,
        valid_until,
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{Unit, UnitPrice, UnitQuantity};
    use crate::providers::common_bridge::{derive_unit_price, parse_quantity};

    #[test]
    pub fn test_parse_quantity() {
        let quantities = vec!["300 g", "380 g", "10 x 55 g", "1,36 kg"];
        let expected = vec![
            UnitQuantity {
                unit: Unit::Gram,
                amount: 300.,
            },
            UnitQuantity {
                unit: Unit::Gram,
                amount: 380.,
            },
            UnitQuantity {
                unit: Unit::Gram,
                amount: 550.,
            },
            UnitQuantity {
                unit: Unit::KiloGram,
                amount: 1.36,
            },
        ];

        assert_eq!(
            quantities.into_iter().flat_map(parse_quantity).collect::<Vec<_>>(),
            expected
        )
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
}
