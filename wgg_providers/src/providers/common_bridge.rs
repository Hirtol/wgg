use std::num::NonZeroU16;

use chrono::{DateTime, Datelike, Utc, Weekday};
use regex::{Regex, RegexBuilder, RegexSet, RegexSetBuilder};

use crate::models::sale_types::{
    NumEuroOff, NumEuroPrice, NumForPrice, NumPercentOff, NumPlusNumFree, NumthPercentOff, SaleType,
};
use crate::models::{CentPrice, SaleValidity, Unit, UnitPrice, UnitQuantity};

#[macro_export]
macro_rules! lazy_re {
    ($name:ident, $regex:expr) => {
        static $name: once_cell::sync::Lazy<Regex> = once_cell::sync::Lazy::new(|| Regex::new($regex).unwrap());
    };
}

#[macro_export]
macro_rules! lazy_re_set {
    ($name:ident, $($regex:expr),*) => {
        static $name: once_cell::sync::Lazy<(regex::RegexSet, Vec<regex::Regex>)> =
            once_cell::sync::Lazy::new(|| $crate::providers::common_bridge::create_regex_set([$($regex,)*]));
    };
}

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
        "kilo" => Some(Unit::KiloGram),
        "Kilo" => Some(Unit::KiloGram),
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

/// Try to parse the given sale label into a proper [SaleType]
pub(crate) fn parse_sale_label(sale_label: &str) -> Option<SaleType> {
    //language=regexp
    lazy_re_set!(
        SALE_RX,
        // Basic `1 + 2 gratis`
        r#"(\d+) \s* \+ \s* (\d+) \s* gratis"#,
        // `2e gratis`
        r#"(\d+) \s* e \s* gratis"#,
        // `50% korting`
        r#"(\d+) \s* % \s* korting"#,
        // `2e halve prijs`
        r#"(\d+) \s* e \s* halve \s* prijs"#,
        // `3 voor €4,50`
        r#"(\d+) \s* voor \s* €? \s* (\d+)(?:[,.](\d+))?"#,
        // `1 euro korting` | `1.50 euro korting`
        r#"(\d+)(?:[,.](\d+))? \s* euro \s* korting"#,
        // `€1.00 korting`
        r#"€ \s* (\d+)(?:[,.](\d+))? \s* korting"#,
        // `NU €4.00`
        r#"NU \s* €(\d+)(?:[,.](\d+))?"#
    );

    let match_idx = SALE_RX.0.matches(sale_label).into_iter().next()?;
    let capture = SALE_RX.1[match_idx].captures(sale_label)?;

    match match_idx {
        0 => {
            // Basic `1 + 2 gratis`
            let (required, free) = (capture.get(1)?, capture.get(2)?);
            let result = NumPlusNumFree {
                required: required.as_str().parse().ok()?,
                free: free.as_str().parse().ok()?,
            };

            Some(SaleType::NumPlusNumFree(result))
        }
        1 => {
            //`2e gratis`
            let required: u16 = capture.get(1)?.as_str().parse().ok()?;
            let result = NumPlusNumFree {
                required: required.checked_sub(1)?.try_into().ok()?,
                free: NonZeroU16::new(1)?,
            };

            Some(SaleType::NumPlusNumFree(result))
        }
        2 => {
            // `50% korting`
            let percent_off: NonZeroU16 = capture.get(1)?.as_str().parse().ok()?;
            let result = NumPercentOff::new(percent_off)?;

            Some(SaleType::NumPercentOff(result))
        }
        3 => {
            // `2e halve prijs`
            let required: NonZeroU16 = capture.get(1)?.as_str().parse().ok()?;
            let result = NumthPercentOff {
                required,
                last_percent_off: NonZeroU16::new(50)?,
            };

            Some(SaleType::NumthPercentOff(result))
        }
        4 => {
            // `3 voor €4,50`
            let required: NonZeroU16 = capture.get(1)?.as_str().parse().ok()?;
            let (integer_part, fractional_part) = (capture.get(2)?, capture.get(3));
            let price = parse_int_fract_price(
                integer_part.as_str().parse().ok()?,
                fractional_part.and_then(|frac| frac.as_str().parse().ok()).unwrap_or(0),
            );
            let result = NumForPrice { required, price };

            Some(SaleType::NumForPrice(result))
        }
        5 | 6 => {
            // `1 euro korting` / `€1.00 korting`
            let (integer_part, fractional_part) = (capture.get(1)?, capture.get(2));
            let price = parse_int_fract_price(
                integer_part.as_str().parse().ok()?,
                fractional_part.and_then(|frac| frac.as_str().parse().ok()).unwrap_or(0),
            );
            let result = NumEuroOff { price_off: price };

            Some(SaleType::NumEuroOff(result))
        }
        7 => {
            // `NU €4.00`
            let (integer_part, fractional_part) = (capture.get(1)?, capture.get(2));
            let price = parse_int_fract_price(
                integer_part.as_str().parse().ok()?,
                fractional_part.and_then(|frac| frac.as_str().parse().ok()).unwrap_or(0),
            );
            let result = NumEuroPrice { price };

            Some(SaleType::NumEuroPrice(result))
        }
        _ => panic!("Wrong index match for sale matching, forgot an update to the match expression?"),
    }
}

pub(crate) fn parse_int_fract_price(integer_part: CentPrice, fractional_part: CentPrice) -> CentPrice {
    (integer_part * 100) + fractional_part
}

/// Return a best estimate sale validity date.
/// Assumes that the sale started at the beginning of `now`'s week (Monday 00:00:00) and will last until the end of `now`'s
/// week (Sunday 23:59:59).
pub(crate) fn get_guessed_sale_validity(now: DateTime<Utc>) -> SaleValidity {
    // We assume a sale is valid until the very end of this week
    let monday = chrono::NaiveDate::from_isoywd_opt(now.iso_week().year(), now.iso_week().week(), Weekday::Mon)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let sunday = chrono::NaiveDate::from_isoywd_opt(now.iso_week().year(), now.iso_week().week(), Weekday::Sun)
        .unwrap()
        .and_hms_opt(23, 59, 59)
        .unwrap();

    let valid_from: DateTime<Utc> = DateTime::from_local(monday, Utc);
    let valid_until: DateTime<Utc> = DateTime::from_local(sunday, Utc);

    SaleValidity {
        valid_from,
        valid_until,
    }
}

/// Create an efficient [RegexSetBuilder] for all the given regexes, alongside individual [Regex] for each respective regex.
/// The regexes are case insensitive and ignore whitespace.
pub(crate) fn create_regex_set(regexes: impl IntoIterator<Item = impl AsRef<str>> + Clone) -> (RegexSet, Vec<Regex>) {
    let set = RegexSetBuilder::new(regexes.clone())
        .case_insensitive(true)
        .ignore_whitespace(true)
        .build()
        .expect("Failed to compile regexes");

    let regexes: Vec<_> = regexes
        .into_iter()
        .map(|pat| {
            RegexBuilder::new(pat.as_ref())
                .case_insensitive(true)
                .ignore_whitespace(true)
                .build()
                .expect("Failed to compile regex")
        })
        .collect();
    (set, regexes)
}

#[cfg(test)]
mod tests {
    use crate::models::{Unit, UnitPrice, UnitQuantity};
    use crate::providers::common_bridge::{derive_unit_price, parse_quantity, parse_sale_label};

    #[test]
    pub fn test_sale_parser() {
        use crate::models::sale_types::*;
        macro_rules! nz {
            ($val:expr) => {
                ::std::num::NonZeroU16::new($val).unwrap()
            };
        }
        let test_cases = [
            "1 + 1 gratis",
            "3 + 4 GRATIS",
            "3e gratis",
            "50% korting",
            "2e halve prijs",
            "3 voor €4,50",
            "4 voor 4.50",
            "2 voor € 2,75",
            "2 voor €3",
            "15 euro korting",
            "1.50 euro korting",
            "€ 1,00 korting",
        ];
        let expected = [
            SaleType::NumPlusNumFree(NumPlusNumFree {
                required: nz!(1),
                free: nz!(1),
            }),
            SaleType::NumPlusNumFree(NumPlusNumFree {
                required: nz!(3),
                free: nz!(4),
            }),
            SaleType::NumPlusNumFree(NumPlusNumFree {
                required: nz!(2),
                free: nz!(1),
            }),
            SaleType::NumPercentOff(NumPercentOff::new(50).unwrap()),
            SaleType::NumthPercentOff(NumthPercentOff {
                required: nz!(2),
                last_percent_off: nz!(50),
            }),
            SaleType::NumForPrice(NumForPrice {
                required: nz!(3),
                price: 450,
            }),
            SaleType::NumForPrice(NumForPrice {
                required: nz!(4),
                price: 450,
            }),
            SaleType::NumForPrice(NumForPrice {
                required: nz!(2),
                price: 275,
            }),
            SaleType::NumForPrice(NumForPrice {
                required: nz!(2),
                price: 300,
            }),
            SaleType::NumEuroOff(NumEuroOff { price_off: 1500 }),
            SaleType::NumEuroOff(NumEuroOff { price_off: 150 }),
            SaleType::NumEuroOff(NumEuroOff { price_off: 100 }),
        ];

        for (to_parse, expected) in test_cases.iter().zip(expected.iter()) {
            assert_eq!(
                parse_sale_label(to_parse).unwrap_or_else(|| panic!("Failed to parse example: {to_parse}")),
                *expected
            );
        }
    }

    #[test]
    pub fn test_parse_quantity() {
        let quantities = ["300 g", "380 g", "10 x 55 g", "1,36 kg"];
        let expected = [
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
