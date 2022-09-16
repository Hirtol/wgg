use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// The price listed as cents.
pub type CentPrice = u32;

#[derive(Serialize, Deserialize, async_graphql::Enum, Hash, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Provider {
    Picnic,
    Jumbo,
}

impl FromStr for Provider {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PICNIC" => Ok(Provider::Picnic),
            "JUMBO" => Ok(Provider::Jumbo),
            _ => anyhow::bail!("Failed to parse provider {}", s),
        }
    }
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct WggAutocomplete {
    pub name: String,
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
pub struct WggSearchProduct {
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
    pub decorators: Vec<WggDecorator>,
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

// ** Full Product **
#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
pub struct WggProduct {
    /// This service's ID for the current product.
    /// Not transferable between [Provider]s
    pub id: String,
    /// The name of the product.
    pub name: String,
    /// Full product description.
    pub description: String,
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
    ///
    /// Can be empty for base ingredients such as cucumbers, for example.
    pub ingredients: Vec<IngredientInfo>,
    /// Denotes the nutritional info, normalised to 100g.
    pub nutritional: Option<NutritionalInfo>,
    /// All information for allergy tags.
    ///
    /// Can be empty if the product has no allergens.
    pub allergy_info: Vec<AllergyTags>,
    /// Denotes all optional bits of information, such as preparation instructions or supplier information.
    ///
    /// These can be useful to add as additional collapsable tabs in the front-end ui.
    pub additional_items: Vec<ItemInfo>,
    /// All decorators describing the object in further detail.
    pub decorators: Vec<WggDecorator>,
    /// The grocery store this item is provided from.
    pub provider: Provider,
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
}

#[derive(Serialize, Deserialize, async_graphql::Enum, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemType {
    PreparationAdvice,
    AdditionalInfo,
    StorageAdvice,
    CountryOfOrigin,
    SafetyWarning,
}

// ** Promotions **
#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
pub struct WggSaleCategory {
    pub id: String,
    pub name: String,
    pub image_urls: Vec<String>,
    /// A potentially limited selection of items, only supported for certain [Provider]s.
    ///
    /// Picnic is one example of such a provider.
    /// Generally recommended to query for more detailed information when needed.
    pub limited_items: Vec<PromotionProduct>,
    pub decorators: Vec<WggDecorator>,
    pub provider: Provider,
}

#[derive(Serialize, Deserialize, async_graphql::Union, Clone, Debug, PartialEq, PartialOrd)]
pub enum PromotionProduct {
    Product(WggSearchProduct),
    ProductId(ProductId),
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, Eq, PartialOrd)]
pub struct ProductId {
    pub id: String,
}

impl From<WggProduct> for WggSearchProduct {
    fn from(product: WggProduct) -> Self {
        WggSearchProduct {
            id: product.id,
            name: product.name,
            full_price: product.full_price,
            display_price: product.display_price,
            unit_quantity: product.unit_quantity,
            unit_price: product.unit_price,
            available: product.available,
            image_url: product.image_urls.into_iter().next(),
            decorators: product.decorators,
            provider: product.provider,
        }
    }
}
