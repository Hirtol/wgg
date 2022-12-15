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
    pub client_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
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
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct ConsentDecisions {
    pub misc_commercial_ads: bool,
    pub misc_commercial_emails: bool,
    pub misc_commercial_messages: bool,
    pub misc_read_advertising_id: bool,
    pub personalized_ranking_consent: bool,
    pub purchases_category_consent: bool,
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
pub struct SearchResult {
    #[serde(rename = "type")]
    pub type_field: String,
    pub id: String,
    pub links: Vec<Link>,
    pub name: String,
    pub items: Vec<SearchItem>,
    pub level: i64,
    pub is_included_in_category_tree: bool,
    pub hidden: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    Promo {
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
        sellable_item_count: i32,
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
pub struct ProductArticle {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub decorators: Vec<Decorator>,
    pub description: Option<Description>,
    pub price_info: PriceInfo,
    pub labels: Labels,
    pub images: Vec<Image>,
    pub max_order_quantity: i32,
    /// Contains the quantity of product, aka `625 grams` or `4-6 pers | 30 mins`
    pub unit_quantity: String,
    pub category_link: Option<String>,
    pub allergies: Allergies,
    /// Contains unstructured info tid-bits like `Na bezorging minimaal 3 dagen vers` or `Binnen 30 minuten op tafel`
    #[serde(default)]
    pub highlights: Vec<Highlight>,
    /// Inserted prior to the description as a single image.
    #[serde(default)]
    pub mood_gallery: Vec<MoodGallery>,
    #[serde(default)]
    pub misc: Vec<Misc>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Description {
    /// Primary text for a description
    pub main: String,
    /// Optional extension for the reader.
    pub extension: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Image {
    pub image_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Labels {
    /// Sustainable brands
    pub brand_tier: Option<Value>,
    #[serde(default)]
    pub status: Vec<Value>,
    #[serde(default)]
    pub characteristics: Vec<Value>,
    /// Contains flavour text like `XL` for apples
    pub size: Option<SizeInfo>,
    /// Contains promo text like `1 + 1 gratis` or `20% korting`
    pub promo: Option<PromoText>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromoText {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SizeInfo {
    pub size: String,
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceInfo {
    pub price: u32,
    pub price_color: Option<String>,
    /// Original price in case the item has a sale, `None` otherwise.
    pub original_price: Option<u32>,
    pub deposit: Option<u32>,
    /// Contains information like `€3.99/kg`
    pub base_price_text: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Allergies {
    #[serde(default)]
    pub allergy_contains: Vec<AllergyContain>,
    #[serde(default)]
    pub allergy_may_contain: Vec<String>,
    pub allergy_text: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllergyContain {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Highlight {
    pub icon: String,
    pub text: String,
    pub action: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoodGallery {
    #[serde(rename = "type")]
    pub type_field: String,
    pub image_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Misc {
    pub header: Header,
    pub body: Body,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Header {
    pub icon: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Body {
    Pml {
        pml_content: PmlContent,
    },
    NutritionalTable {
        nutritional_table: NutritionalTable,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PmlContent {
    pub pml_version: String,
    pub component: PmlComponent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PmlComponent {
    Stack(PmlStack),
    RichText(PmlChildren),
    #[serde(other)]
    Other,
}

/// Tends to contain additional info such as country of origin, company,
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PmlStack {
    pub axis: String,
    pub spacing: String,
    #[serde(default)]
    pub children: Vec<PmlChildren>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PmlChildren {
    pub text_type: String,
    pub text_alignment: String,
    pub markdown: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NutritionalTable {
    pub default_unit: String,
    pub values: Vec<NutritionalValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NutritionalValue {
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub sub_values: Vec<NutritionalSubValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NutritionalSubValue {
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub sub_values: Vec<NutritionalSubValue>,
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
    pub total_deposit: Option<u32>,
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
    /// Only available when adding/removing a product to the cart which has an associated sale.
    pub promo_progress: Option<PromoProgress>,
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

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromoProgress {
    /// The amount of items needed to complete this
    pub total: u32,
    pub completed_savings: Option<u32>,
    /// For example "2e halve prijs" or "1+1 gratis"
    pub label: String,
    pub description: Option<String>,
    pub product_ids: Vec<String>,
    pub promotion_id: String,
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
