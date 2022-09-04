use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

// ** LOGIN **

#[derive(Serialize, Deserialize)]
pub(crate) struct LoginRequest {
    pub key: String,
    pub secret: String,
    pub client_id: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct LoginResponse {
    pub user_id: String,
    pub second_factor_authentication_required: bool,
    pub show_second_factor_authentication_intro: bool,
}

// ** USER INFO **

#[derive(Serialize, Deserialize, Debug)]
pub struct UserInfo {
    pub user_id: String,
    pub firstname: String,
    pub lastname: String,
    pub address: Address,
    pub phone: String,
    pub contact_email: String,
    pub feature_toggles: Vec<String>,
    pub push_subscriptions: Vec<Subscriptions>,
    pub subscriptions: Vec<Subscriptions>,
    pub customer_type: String,
    pub household_details: HouseholdDetails,
    pub check_general_consent: bool,
    pub placed_order: bool,
    pub received_delivery: bool,
    pub total_deliveries: i64,
    pub completed_deliveries: i64,
    pub consent_decisions: ConsentDecisions,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConsentDecisions {
    #[serde(rename = "MISC_COMMERCIAL_ADS")]
    pub misc_commercial_ads: bool,
    #[serde(rename = "MISC_COMMERCIAL_EMAILS")]
    pub misc_commercial_emails: bool,
    #[serde(rename = "MISC_COMMERCIAL_MESSAGES")]
    pub misc_commercial_messages: bool,
    #[serde(rename = "MISC_READ_ADVERTISING_ID")]
    pub misc_read_advertising_id: bool,
    #[serde(rename = "PERSONALIZED_RANKING_CONSENT")]
    pub personalized_ranking_consent: bool,
    #[serde(rename = "PURCHASES_CATEGORY_CONSENT")]
    pub purchases_category_consent: bool,
    #[serde(rename = "WEEKLY_COMMERCIAL_EMAILS")]
    pub weekly_commercial_emails: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HouseholdDetails {
    pub adults: i64,
    pub children: i64,
    pub cats: i64,
    pub dogs: i64,
    pub author: String,
    pub last_edit_ts: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Subscriptions {
    pub list_id: String,
    pub subscribed: bool,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Address {
    pub house_number: i64,
    pub house_number_ext: String,
    pub postcode: String,
    pub street: String,
    pub city: String,
}

// ** SEARCH **

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    #[serde(rename = "type")]
    pub type_field: String,
    pub id: String,
    pub links: Vec<Link>,
    pub name: String,
    pub items: Vec<SearchItem>,
    pub level: i64,
    #[serde(rename = "is_included_in_category_tree")]
    pub is_included_in_category_tree: bool,
    pub hidden: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    #[serde(rename = "type")]
    pub type_field: String,
    pub href: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SearchItem {
    SingleArticle(SingleArticle),
    ItemSuggestionDialog,
    #[serde(other)]
    Other,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SingleArticle {
    pub id: String,
    #[serde(default)]
    pub decorators: Vec<Decorator>,
    pub name: String,
    pub display_price: u32,
    pub price: Option<u32>,
    #[serde(default)]
    pub image_id: String,
    pub max_count: u32,
    #[serde(default)]
    pub unit_quantity: String,
    pub unit_quantity_sub: Option<String>,
    pub tags: Vec<Value>,
}

// ** Decorator **

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Decorator {
    BasePrice {
        base_price_text: String,
    },
    FreshLabel {
        period: String,
    },
    Label {
        text: String,
    },
    Price {
        display_price: u32,
    },
    BackgroundImage {
        image_ids: Vec<String>,
        height_percent: i32,
    },
    Banners {
        height_percentage: i32,
        banners: Vec<Banner>,
    },
    UnitQuantity {
        unit_quantity_text: String,
    },
    ValidityLabel {
        valid_until: chrono::NaiveDate,
    },
    TitleStyle {
        styles: Vec<Style>,
    },
    MoreButton {
        link: Link,
        images: Vec<String>,
        sellable_item_count: u32,
    },
    Unavailable {
        reason: UnavailableReason,
        #[serde(default)]
        replacements: Vec<SingleArticle>,
        explanation: Explanation,
    },
    ArticleDeliveryIssues {
        issues: Vec<Issue>,
    },
    Quantity {
        quantity: i64,
    },
    #[serde(other)]
    Other,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Banner {
    pub banner_id: String,
    pub image_id: String,
    pub display_time: String,
    pub description: String,
    pub position: String,
    pub reference: Option<Reference>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reference {
    pub target: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Style {
    pub position: Position,
    pub color: String,
    pub style: String,
    pub priority: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub start_index: i64,
    pub length: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UnavailableReason {
    Available,
    OutOfAssortment,
    OutOfSeason,
    TemporarilyUnavailable,
    Unknown,
    #[serde(other)]
    Other,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Explanation {
    pub short_explanation: String,
    pub long_explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Issue {
    pub article_id: String,
    pub price: i64,
    pub quantity: i64,
    pub reason: ArticleIssueReasonType,
    pub resolution: IssueResolution,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IssueResolution {
    None,
    FreeInBasket,
    Refund,
    Substituted {
        substitutions: Vec<OrderArticle>,
        refunded: bool,
    },
    Unsupported,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ArticleIssueReasonType {
    ProductNotShipped,
    ProductDamaged,
    ProductNotRequested,
    ProductLowQuality,
    ProductAgeRequirementNotMet,
    ProductAbsent,
    ProductPricePromiseBroken,
    ProductSubstituted,
    ProductNotProcessed,
    Unsupported,
}

// ** Suggestions **

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Suggestion {
    #[serde(rename = "type")]
    pub type_field: String,
    pub id: String,
    pub links: Vec<Link>,
    pub suggestion: String,
}

// ** Product Info **

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Product {
    pub current_count: i64,
    pub max_count: i64,
    pub price: i64,
    pub name: String,
    pub fresh_label: FreshLabel,
    pub product_id: String,
    pub unit_quantity_sub: String,
    pub deposit: i64,
    pub image_id: String,
    pub unit_quantity: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubItemDetails {
    pub id: String,
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemDetails {
    pub id: String,
    pub title: String,
    pub items: Vec<SubItemDetails>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NutritionalValue {
    pub name: String,
    pub value: String,
    pub gda_percentage: String,
    #[serde(default)]
    pub sub_values: Vec<SubValue>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubValue {
    pub name: String,
    pub value: String,
    pub gda_percentage: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FreshLabel {
    pub unit: String,
    pub number: i32,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DietTags {
    pub name: String,
    pub color: String,
    pub description: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductDetails {
    pub id: String,
    pub decorators: Vec<Decorator>,
    pub name: String,
    pub display_price: i32,
    pub price: i32,
    pub image_id: String,
    pub max_count: i32,
    pub unit_quantity: String,
    pub unit_quantity_sub: String,
    pub tags: Vec<DietTags>,
    pub product_id: String,
    pub description: String,
    pub canonical_name: Option<String>,
    pub image_ids: Vec<String>,
    pub fresh_label: FreshLabel,
    pub nutritional_values: Vec<NutritionalValue>,
    pub ingredients_blob: String,
    pub additional_info: String,
    pub label_holder: String,
    pub items: Vec<ItemDetails>,
    pub nutritional_info_unit: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductResult {
    pub product_details: ProductDetails,
    pub products: Vec<Product>,
}

// ** Images **

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageSize {
    Tiny,
    Small,
    Medium,
    Large,
    ExtraLarge,
}

impl Display for ImageSize {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageSize::Tiny => f.write_str("tiny"),
            ImageSize::Small => f.write_str("small"),
            ImageSize::Medium => f.write_str("medium"),
            ImageSize::Large => f.write_str("large"),
            ImageSize::ExtraLarge => f.write_str("extra-large"),
        }
    }
}

// ** My Store **
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MyStore {
    #[serde(rename = "type")]
    pub type_field: String,
    /// Contains all global categories (think Promotions, Recipes)
    pub catalog: Vec<Catalog>,
    pub content: Vec<Value>,
    pub first_time_user: bool,
    pub landing_page_hint: String,
    pub id: String,
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Catalog {
    pub id: String,
    pub name: String,
    pub items: Vec<Category>,
    pub level: i64,
    pub is_included_in_category_tree: bool,
    pub hidden: bool,
    #[serde(default)]
    pub decorators: Vec<Decorator>,
    pub links: Vec<Link>,
    pub image_id: Option<String>,
    pub header_image_id: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub items: Vec<SubCategory>,
    pub level: i64,
    pub is_included_in_category_tree: bool,
    pub hidden: bool,
    #[serde(default)]
    pub decorators: Vec<Decorator>,
    #[serde(default)]
    pub links: Vec<Link>,
    pub image_id: Option<String>,
    pub header_image_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubCategory {
    Category(Category),
    SingleArticle(SingleArticle),
    #[serde(other)]
    Other,
}

// ** Shopping Cart **

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    #[serde(default)]
    pub items: Vec<OrderLine>,
    #[serde(default)]
    pub delivery_slots: Vec<DeliverySlot>,
    /// Only available if a timeslot has been selected for this particular order.
    pub selected_slot: Option<SelectedSlot>,
    pub slot_selector_message: Option<Value>,
    /// Only available in the shopping cart overview of the Order
    pub total_count: Option<i64>,
    pub total_price: i64,
    pub checkout_total_price: i64,
    pub total_savings: i64,
    /// Only available once the total price exceeds the minimum (at the moment, 35,- euros) quantity.
    pub total_deposit: Option<i64>,
    /// Only available once the order has been placed.
    pub cancellable: Option<bool>,
    /// Only available once the order has been placed, and cancelled.
    pub cancellation_time: Option<String>,
    /// Only available once the order has been placed.
    pub creation_time: Option<String>,
    /// Only available once the order has been placed.
    pub status: Option<DeliveryStatus>,
    #[serde(default)]
    pub deposit_breakdown: Vec<DepositBreakdown>,
    pub decorator_overrides: HashMap<String, Vec<Decorator>>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderLine {
    pub id: String,
    pub items: Vec<OrderArticle>,
    pub display_price: i64,
    pub price: i64,
    #[serde(default)]
    pub decorators: Vec<Decorator>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderArticle {
    #[serde(rename = "type")]
    pub type_field: String,
    pub id: String,
    pub name: String,
    pub image_ids: Vec<String>,
    pub unit_quantity: String,
    pub unit_quantity_sub: Option<String>,
    pub price: i64,
    pub max_count: i64,
    pub perishable: bool,
    pub tags: Vec<Value>,
    pub decorators: Vec<Decorator>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectedSlot {
    pub slot_id: String,
    pub state: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepositBreakdown {
    #[serde(rename = "type")]
    pub type_field: String,
    value: i64,
    count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderStatus {
    pub checkout_status: CheckoutStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CheckoutStatus {
    Abandoned,
    Failed,
    Finished,
    Ongoing,
    Unknown,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ModifyCartProduct<'a> {
    /// The product to add or remove.
    pub product_id: &'a crate::ProductId,
    /// The amount of the provided product to add/remove.
    pub count: u32,
}

// ** Deliveries **
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Delivery {
    pub delivery_id: String,
    pub creation_time: DateTime<Utc>,
    pub slot: DeliverySlot,
    pub eta2: TimeWindow,
    pub status: DeliveryStatus,
    pub delivery_time: TimeWindow,
    pub id: String,
    pub decorators: Vec<Decorator>,
    pub orders: Vec<Order>,
    pub returned_containers: Vec<ReturnedContainer>,
    pub parcels: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReturnedContainer {
    #[serde(rename = "type")]
    pub type_field: String,
    pub localized_name: String,
    pub quantity: i64,
    pub price: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliverySlotQuery {
    pub delivery_slots: Vec<DeliverySlot>,
    pub selected_slot: SelectedSlot,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliverySlot {
    pub slot_id: String,
    pub hub_id: String,
    pub fc_id: String,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub cut_off_time: DateTime<Utc>,
    pub is_available: bool,
    pub selected: bool,
    pub reserved: bool,
    pub minimum_order_value: i64,
    pub unavailability_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartialDelivery {
    pub delivery_id: String,
    pub creation_time: DateTime<Utc>,
    pub slot: DeliverySlot,
    pub eta2: TimeWindow,
    pub status: DeliveryStatus,
    pub delivery_time: TimeWindow,
    /// Contains partial orders, more detailed information needs to be queried separately.
    pub orders: Vec<PartialOrder>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeWindow {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartialOrder {
    pub id: String,
    pub creation_time: DateTime<Utc>,
    pub total_price: i64,
    pub status: DeliveryStatus,
    pub cancellation_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeliveryStatus {
    Current,
    Completed,
    Cancelled,
    #[serde(other)]
    Other,
}

impl Display for DeliveryStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeliveryStatus::Current => f.write_str("CURRENT"),
            DeliveryStatus::Completed => f.write_str("COMPLETED"),
            DeliveryStatus::Cancelled => f.write_str("CANCELLED"),
            _ => f.write_str("OTHER"),
        }
    }
}
