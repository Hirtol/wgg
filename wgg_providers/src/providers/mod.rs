use crate::{OffsetPagination, Provider, Result, WggAutocomplete, WggSearchProduct};
use std::borrow::Cow;

mod common_bridge;
mod jumbo_bridge;
mod picnic_bridge;

use crate::models::{WggProduct, WggSaleCategory, WggSaleGroupComplete};
pub(crate) use jumbo_bridge::*;
pub(crate) use picnic_bridge::*;

pub trait StaticProviderInfo {
    /// The associated [Provider] for this bridge implementation
    fn provider() -> Provider
    where
        Self: Sized;

    /// A direct link to a SVG displaying the provider's logo.
    fn logo_url() -> Cow<'static, str>
    where
        Self: Sized;
}

#[async_trait::async_trait]
pub trait ProviderInfo: StaticProviderInfo {
    fn provider(&self) -> Provider;

    fn logo_url(&self) -> Cow<'static, str>;

    /// Perform an autocomplete match for the provided query.
    ///
    /// Some APIs will perform a network call, whilst others will do in-process filtering to provide a list of terms.
    async fn autocomplete(&self, query: &str) -> Result<Vec<WggAutocomplete>>;

    /// Perform a search for the provided term, and return the pages.
    ///
    /// # Arguments
    /// * `offset` - Can be `None` for APIs which don't support pagination, but should be used for others which do.
    async fn search(&self, query: &str, offset: Option<u32>) -> Result<OffsetPagination<WggSearchProduct>>;

    /// Retrieve the full product info for the provided `product_id`.
    async fn product(&self, product_id: &str) -> Result<WggProduct>;

    /// Retrieve all current promotions
    async fn promotions(&self) -> Result<Vec<WggSaleCategory>>;

    /// Retrieve a specific promotion
    async fn promotions_sublist(&self, sublist_id: &str) -> Result<WggSaleGroupComplete>;
}
