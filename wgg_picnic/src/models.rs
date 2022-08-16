use serde::{Deserialize, Serialize};
use serde_json::Value;

// ** LOGIN STUFF **

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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    #[serde(rename = "type")]
    pub type_field: String,
    pub href: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SearchItem {
    SingleArticle(SingleArticle),
    ItemSuggestionDialog,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SingleArticle {
    pub id: String,
    pub decorators: Vec<Decorator>,
    pub name: String,
    pub display_price: i64,
    pub price: Option<i64>,
    pub image_id: String,
    pub max_count: i64,
    pub unit_quantity: String,
    pub unit_quantity_sub: String,
    pub tags: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Decorator {
    #[serde(rename = "type")]
    pub type_field: String,
    pub period: Option<String>,
    pub unit_quantity_text: Option<String>,
    #[serde(default)]
    pub styles: Vec<Style>,
    pub text: Option<String>,
    pub display_price: Option<i64>,
    pub valid_until: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Style {
    pub position: Position,
    pub color: String,
    pub style: String,
    pub priority: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub start_index: i64,
    pub length: i64,
}

// ** Suggestions **

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Suggestion {
    #[serde(rename = "type")]
    pub type_field: String,
    pub id: String,
    pub links: Vec<Link>,
    pub suggestion: String,
}
