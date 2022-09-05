use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The price listed as cents.
pub type CentPrice = u32;

#[derive(Serialize, Deserialize, async_graphql::Enum, Hash, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Provider {
    Picnic,
    Jumbo,
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Autocomplete {
    pub name: String,
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
pub struct SearchProduct {
    pub id: String,
    pub name: String,
    /// The full price of an article, ignoring any sales
    pub full_price: CentPrice,
    /// The present display price (taking into account active sales).
    pub display_price: CentPrice,
    /// The amount of weight/liters/pieces this product represents.
    pub unit_quantity: UnitQuantity,
    pub unit_price: Option<UnitPrice>,
    /// A small check to see if the current item is unavailable.
    ///
    /// `decorators` might contains more information as to the nature of the disruption.
    pub available: bool,
    /// Direct URL to product image.
    pub image_url: Option<String>,
    pub decorators: Vec<Decorator>,
    /// The grocery store this item is provided from.
    pub provider: Provider,
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
pub enum Decorator {
    FreshLabel(FreshLabel),
    SaleLabel(SaleLabel),
    SaleValidity(SaleValidity),
    Unavailable(UnavailableItem),
    PrepTime(PrepTime),
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

/// If the item is unavailable
#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
pub struct UnavailableItem {
    pub reason: UnavailableReason,
    pub explanation_short: Option<String>,
    pub explanation_long: Option<String>,
    /// Lists replacements if the store has suggested any.
    ///
    /// Some stores won't support this functionality, and this would therefore remain empty.
    pub replacements: Vec<SearchProduct>,
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PrepTime {
    pub time_minutes: u32,
}

#[derive(Serialize, Deserialize, async_graphql::Enum, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnavailableReason {
    OutOfAssortment,
    OutOfSeason,
    TemporarilyUnavailable,
    Unknown,
}

// ** Full Product **
#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
pub struct Product {
    /// This service's ID for the current product.
    /// Not transferable between [Provider]s
    pub id: String,
    /// The name of the product.
    pub name: String,
    /// Full product description.
    pub description: String,
    /// Any additional info relevant for the consumer.
    pub additional_info: Option<String>,
    /// The full price of an article, ignoring any sales
    pub full_price: CentPrice,
    /// The present display price (taking into account active sales).
    pub display_price: CentPrice,
    /// The amount of weight/liters/pieces this product represents.
    pub unit_quantity: UnitQuantity,
    pub unit_price: Option<UnitPrice>,
    /// A small check to see if the current item is unavailable.
    ///
    /// `decorators` might contains more information as to the nature of the disruption.
    pub available: bool,
    /// Direct URL to product image.
    pub image_urls: Vec<String>,
    /// All ingredients in a structured format.
    pub ingredients: Vec<IngredientInfo>,
    /// Denotes the nutritional info, normalised to 100g.
    pub nutritional: Vec<NutritionalInfo>,
    /// All information for allergy tags.
    pub allergy_info: Vec<AllergyTags>,
    /// Denotes all optional bits of information, such as preparation instructions or supplier information.
    ///
    /// These can be useful to add as additional collapsable tabs in the front-end ui.
    pub additional_items: Vec<ItemInfo>,
    /// All decorators describing the object in further detail.
    pub decorators: Vec<Decorator>,
    /// The grocery store this item is provided from.
    pub provider: Provider,
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
pub struct IngredientInfo {
    name: String,
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
pub struct NutritionalInfo {
    name: String,
    value: String,
    sub_values: Vec<SubNutritionalInfo>,
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
pub struct SubNutritionalInfo {
    name: String,
    value: String,
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
pub struct AllergyTags {
    pub name: String,
}

/// Contains additional information relevant for an item.
///
/// Examples include: Preparation instructions, Supplier info
#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
pub struct ItemInfo {
    title: String,
    item_type: Option<ItemType>,
    text: String,
}

#[derive(Serialize, Deserialize, async_graphql::Enum, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemType {
    PreparationAdvice,
    AdditionalInfo,
    SupplierInfo,
}
