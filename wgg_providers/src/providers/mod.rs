use crate::{Autocomplete, OffsetPagination, Provider, Result, SearchProduct};
use std::borrow::Cow;

mod common_bridge;
mod jumbo_bridge;
mod picnic_bridge;

use crate::models::{Product, PromotionCategory};
pub(crate) use jumbo_bridge::*;
pub(crate) use picnic_bridge::*;

#[async_trait::async_trait]
pub trait ProviderInfo {
    fn provider() -> Provider
    where
        Self: Sized;

    fn logo_url(&self) -> Cow<'static, str>;

    /// Perform an autocomplete match for the provided query.
    ///
    /// Some APIs will perform a network call, whilst others will do in-process filtering to provide a list of terms.
    async fn autocomplete(&self, query: &str) -> Result<Vec<Autocomplete>>;

    /// Perform a search for the provided term, and return the pages.
    ///
    /// # Arguments
    /// * `offset` - Can be `None` for APIs which don't support pagination, but should be used for others which do.
    async fn search(&self, query: &str, offset: Option<u32>) -> Result<OffsetPagination<SearchProduct>>;

    /// Retrieve the full product info for the provided `product_id`.
    async fn product(&self, product_id: &str) -> Result<Product>;

    /// Retrieve all current promotions
    async fn promotions(&self) -> Result<Vec<PromotionCategory>>;

    /// Retrieve a specific promotion
    async fn promotions_sublist(&self, sublist_id: &str) -> Result<OffsetPagination<SearchProduct>>;
}
