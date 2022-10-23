use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
mod product;
mod providers;
mod sale;
mod search_product;

pub use product::*;
pub use providers::*;
pub use sale::*;
pub use search_product::*;

/// The price listed as cents.
pub type CentPrice = u32;

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct WggAutocomplete {
    pub name: String,
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnitPrice {
    pub unit: Unit,
    pub price: CentPrice,
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
pub struct UnitQuantity {
    pub unit: Unit,
    pub amount: f64,
}

impl Default for UnitQuantity {
    fn default() -> Self {
        UnitQuantity {
            unit: Unit::Piece,
            amount: 1.0,
        }
    }
}

#[derive(Serialize, Deserialize, async_graphql::Enum, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Unit {
    Piece,
    Liter,
    MilliLiter,
    KiloGram,
    Gram,
}

#[derive(Serialize, Deserialize, async_graphql::Union, Clone, Debug, PartialEq, PartialOrd)]
#[serde(tag = "type")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WggDecorator {
    FreshLabel(FreshLabel),
    SaleLabel(SaleLabel),
    SaleValidity(SaleValidity),
    SaleDescription(SaleDescription),
    Unavailable(UnavailableItem),
    PrepTime(PrepTime),
    NumberOfServings(NumberOfServings),
    MoreButton(MoreButton),
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FreshLabel {
    pub days_fresh: u32,
}

/// Describes the type of sale that applies to the attached object.
///
/// Think of "1 + 1 Free", or "50% off".
#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SaleLabel {
    pub text: String,
}

/// Until what date (inclusive) the attached sale is valid.
#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SaleValidity {
    pub valid_from: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
}

/// A subtitle for a particular sale.
#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SaleDescription {
    pub text: String,
}

/// If the item is unavailable
#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
pub struct UnavailableItem {
    pub reason: UnavailableReason,
    pub explanation_short: Option<String>,
    pub explanation_long: Option<String>,
    /// Lists replacements if the store has suggested any.
    ///
    /// Some stores won't support this functionality, and this would therefore remain empty.
    pub replacements: Vec<WggSearchProduct>,
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PrepTime {
    pub time_minutes: u32,
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NumberOfServings {
    pub amount: u32,
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MoreButton {
    pub images: Vec<String>,
}

#[derive(Serialize, Deserialize, async_graphql::Enum, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnavailableReason {
    OutOfAssortment,
    OutOfSeason,
    TemporarilyUnavailable,
    Unknown,
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, Eq, PartialOrd)]
pub struct PriceInfo {
    /// The present display price (taking into account active sales).
    pub display_price: CentPrice,
    /// The full price of an article, ignoring any sales
    pub original_price: CentPrice,
    pub unit_price: Option<UnitPrice>,
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, Eq, PartialOrd)]
pub struct IngredientInfo {
    pub name: String,
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, Eq, PartialOrd)]
pub struct NutritionalInfo {
    /// For what unit (e.g, `per 100g`) these items are valid.
    pub info_unit: String,
    pub items: Vec<NutritionalItem>,
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, Eq, PartialOrd)]
pub struct NutritionalItem {
    pub name: String,
    pub value: String,
    pub sub_values: Vec<SubNutritionalItem>,
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, Eq, PartialOrd)]
pub struct SubNutritionalItem {
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, Eq, PartialOrd)]
pub struct AllergyTags {
    pub name: String,
    pub contains: AllergyType,
}

#[derive(Serialize, Deserialize, async_graphql::Enum, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AllergyType {
    Contains,
    MayContain,
}

/// Contains additional information relevant for an item.
///
/// Examples include: Preparation instructions, Supplier info
#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, Eq, PartialOrd)]
pub struct ItemInfo {
    pub item_type: ItemType,
    pub text: String,
    pub text_type: TextType,
}

#[derive(Serialize, Deserialize, async_graphql::Enum, Copy, Clone, Debug, PartialEq, Eq, PartialOrd)]
pub enum TextType {
    PlainText,
    Markdown,
}

#[derive(Serialize, Deserialize, async_graphql::Enum, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemType {
    PreparationAdvice,
    AdditionalInfo,
    StorageAdvice,
    CountryOfOrigin,
    SafetyWarning,
}