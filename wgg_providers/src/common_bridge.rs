use crate::models::{CentPrice, Unit, UnitPrice, UnitQuantity};

/// Try to parse the provided price in the format `l` or `kg` as a [crate::models::Unit].
///
/// Invalid input will return [None]
pub(crate) fn parse_unit_component(unit: &str) -> Option<Unit> {
    match unit {
        "l" => Some(Unit::Liter),
        "liter" => Some(Unit::Liter),
        "ml" => Some(Unit::MilliLiter),
        "kg" => Some(Unit::KiloGram),
        "g" => Some(Unit::Gram),
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
        let (quantity, unit) = (whitespaces.next()?, whitespaces.next()?);
        // TODO: This cuts of `1.5 liter` at the moment!
        let multiplier: f64 = multiplier.parse().ok()?;
        let quantity: f64 = quantity.parse().ok()?;
        let unit = parse_unit_component(unit)?;

        UnitQuantity {
            unit,
            amount: (quantity * multiplier).round(),
        }
        .into()
    } else {
        let mut whitespaces = quantity.split_whitespace();
        let (quantity, unit) = (whitespaces.next()?, whitespaces.next()?);
        // TODO: This cuts of `1.5 liter` at the moment!
        let quantity: f64 = quantity.parse().ok()?;
        let unit = parse_unit_component(unit)?;

        UnitQuantity { unit, amount: quantity }.into()
    }
}

/// Try to derive a unit price from the unit quantity and display price.
///
/// Preferably one would first use [parse_unit_price], but this function is available as a fallback.
pub(crate) fn derive_unit_price(unit_quantity: &UnitQuantity, display_price: CentPrice) -> Option<UnitPrice> {
    let (normalised_quantity, normalised_unit) = match unit_quantity.unit {
        Unit::Gram => ((unit_quantity.amount as f64 / 1000.), Unit::KiloGram),
        Unit::MilliLiter => ((unit_quantity.amount as f64 / 1000.), Unit::Liter),
        _ => (unit_quantity.amount, unit_quantity.unit),
    };

    UnitPrice {
        unit: normalised_unit,
        price: (display_price as f64 / normalised_quantity).round() as CentPrice,
    }
    .into()
}

#[cfg(test)]
mod tests {
    use crate::common_bridge::parse_quantity;
    use crate::models::{Unit, UnitQuantity};

    #[test]
    pub fn test_parse_quantity() {
        let quantities = vec!["300 g", "380 g", "10 x 55 g"];
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
        ];

        assert_eq!(
            quantities.into_iter().flat_map(parse_quantity).collect::<Vec<_>>(),
            expected
        )
    }
}
