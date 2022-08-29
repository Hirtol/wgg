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
    pub promotions: Vec<Promotion>,
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

// ** Partial Products **

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
    /// Contains the information regarding the category of this product.
    ///
    /// Only available when queried from the `/products` endpoint.
    #[serde(flatten)]
    pub category: Option<CategoryInfo>,
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
#[serde(rename_all = "camelCase")]
pub struct CategoryInfo {
    pub top_level_category: String,
    pub top_level_category_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProductType {
    Product,
    PartOfRetailSet,
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

// ** Full Product **

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FullProductResponse {
    pub product: ProductInfo,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductInfo {
    pub data: Product,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    pub id: ProductId,
    pub title: String,
    pub quantity_options: Vec<QuantityOption>,
    pub prices: Prices,
    pub available: bool,
    pub product_type: ProductType,
    pub quantity: Option<String>,
    pub image_info: ImageInfo,
    pub top_level_category: String,
    pub top_level_category_id: String,
    pub sample: bool,
    pub availability: Availability,
    pub has_related_products: bool,
    pub nutritional_information: Vec<NutritionalInformation>,
    pub number_of_servings: Option<String>,
    pub regulated_title: Option<String>,
    #[serde(default)]
    pub ingredient_info: Vec<IngredientInfo>,
    pub allergy_text: Option<String>,
    pub allergy_info: Option<AllergyInfo>,
    pub usage_and_safety_info: Option<UsageAndSafetyInfo>,
    pub origin_info: Option<OriginInfo>,
    pub brand_info: Option<BrandInfo>,
    pub promotion: Option<ProductPromotion>,
    /// Will only show up if `product_type == ProductType::RetailSet`.
    pub retail_set_products: Option<Vec<PartialProduct>>,
    #[serde(default)]
    pub sticker_badges: Vec<String>,
    pub badge_description: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NutritionalInformation {
    pub product_title: String,
    pub nutritional_guidelines: ProductNutritionalGuidelines,
    pub nutritional_data: NutritionalData,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductNutritionalGuidelines {
    #[serde(default)]
    pub entries: Vec<ProductNutritionalGuideline>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductNutritionalGuideline {
    pub name: String,
    pub percentage: Option<String>,
    pub quantity: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NutritionalData {
    pub entries: Vec<ProductNutrition>,
    pub portion_size: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductNutrition {
    pub name: String,
    pub value_per100g: String,
    pub value_per_portion: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngredientInfo {
    pub product_title: String,
    pub ingredients: Vec<Ingredient>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ingredient {
    pub name: String,
    pub contains_allergens: bool,
    #[serde(default)]
    pub highlights: Vec<HighlightRange>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HighlightRange {
    pub length: i64,
    pub offset: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllergyInfo {
    pub allergy_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageAndSafetyInfo {
    pub storage_type: String,
    pub safety_warning: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OriginInfo {
    pub fishing_area: Option<String>,
    pub country_of_origin: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrandInfo {
    pub manufacturer_address: String,
    pub web_address: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductPromotion {
    pub id: String,
    pub name: String,
    pub label: String,
    #[serde(rename = "image")]
    pub image_url: String,
    pub validity_period: String,
    pub summary: String,
    pub offline: bool,
    pub tags: Vec<Tag>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub text: String,
}

// ** Search **
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoCompleteResponse {
    pub autocomplete: Autocomplete,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Autocomplete {
    /// Contains *all* the various terms Jumbo deems as part of potential auto-complete features.
    /// Filtering is a client-side task!
    /// This is an incredibly stupid endpoint...
    pub data: Vec<String>,
    pub total: u32,
}

// ** Auth **
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserResponse {
    pub user: User,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub data: UserData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserData {
    pub first_name: String,
    pub suffix: String,
    pub last_name: String,
    pub identifier: String,
    pub gender: String,
    pub phone_number: PhoneNumber,
    pub email: String,
    pub address: Address,
    pub allow_profiling: bool,
    pub communication: Communication,
    #[serde(rename = "type")]
    pub user_type: String,
    pub transactional_optin: bool,
    pub commercial_optin: bool,
    pub social_list_optin: bool,
    pub active: bool,
    pub store: Store,
    pub has_placed_order: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhoneNumber {
    pub number: String,
    pub country_code: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    pub country: String,
    pub country_code: String,
    pub postalcode: String,
    pub address_line: String,
    pub house_number: String,
    pub city: String,
    pub house_number_addition: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Store {
    pub id: String,
    pub complex_number: String,
    pub name: String,
    pub r#type: String,
    pub street_address: String,
    pub city_name: String,
    pub zip_code: String,
    pub longitude: f64,
    pub latitude: f64,
    pub new_location: bool,
    pub is_last_order_store: bool,
    pub opening_times: Vec<OpeningTimes>,
    pub phone_number: String,
    pub facilities: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpeningTimes {
    pub date: i64,
    pub time: String,
    pub today: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Communication {
    #[serde(rename = "MobileApp")]
    pub mobile_app: bool,
    #[serde(rename = "SMS")]
    pub sms: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}
