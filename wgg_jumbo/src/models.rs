use crate::ids::{ProductId, PromotionId, RuntimeId, TabId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

// ** Promotions **

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromotionTabs {
    pub tabs: Vec<Tab>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tab {
    pub id: TabId,
    pub title: String,
    pub short_title: String,
    pub runtimes: Vec<Runtime>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Runtime {
    pub id: RuntimeId,
    pub title: String,
    pub short_title: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SortedByQuery {
    Date,
    Product,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromotionGroup {
    pub categories: Vec<PromotionCategory>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromotionCategory {
    pub title: String,
    pub promotions: Vec<Promotion>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Promotion {
    pub id: PromotionId,
    pub title: String,
    pub description: String,
    pub primary_badges: Vec<PromotionBadge>,
    pub secondary_badges: Vec<PromotionBadge>,
    pub tags: Vec<String>,
    pub subtitle: Option<String>,
    pub image: PromotionImage,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub products: Vec<ProductId>,
    pub disclaimer: Option<String>,
    pub offline_text: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromotionBadge {
    pub image: PromotionImage,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromotionImage {
    pub url: String,
    pub relative_path: String,
}

// ** Products **

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductList {
    pub products: ProductsPage,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductsPage {
    pub data: Vec<PartialProduct>,
    pub total: u32,
    pub offset: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartialProduct {
    pub id: ProductId,
    pub title: String,
    pub quantity_options: Vec<QuantityOption>,
    pub prices: Prices,
    pub available: bool,
    pub product_type: ProductType,
    pub image_info: ImageInfo,
    pub top_level_category: String,
    pub top_level_category_id: String,
    pub sample: bool,
    pub availability: Availability,
    pub quantity: Option<String>,
    /// Will only show up if `product_type == ProductType::RetailSet`.
    pub retail_set_products: Option<Vec<PartialProduct>>,
    #[serde(default)]
    pub sticker_badges: Vec<String>,
    pub badge_description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProductType {
    Product,
    RetailSet,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuantityOption {
    pub default_amount: u32,
    pub minimum_amount: u32,
    pub amount_step: u32,
    pub unit: Unit,
    pub maximum_amount: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    Kg,
    Liter,
    Piece,
    Pieces,
    #[serde(other)]
    Other,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Prices {
    pub price: Price,
    pub promotional_price: Option<Price>,
    pub unit_price: Option<UnitPrice>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Price {
    pub currency: String,
    pub amount: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitPrice {
    pub unit: String,
    pub price: Price,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageInfo {
    pub primary_view: Vec<ProductImage>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductImage {
    pub url: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Availability {
    pub sku: Option<String>,
    pub availability: String,
    pub reason: Option<String>,
    pub label: Option<String>,
}
