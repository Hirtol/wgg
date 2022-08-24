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
