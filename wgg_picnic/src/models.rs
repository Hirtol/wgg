use serde::{Deserialize, Serialize};

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
