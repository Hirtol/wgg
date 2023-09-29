use crate::error::Result;
use crate::models::{ProductIdRef, Provider, ProviderMetadata, WggAutocomplete, WggSearchProduct};
use crate::models::{WggProduct, WggSaleCategory, WggSaleGroupComplete};
use crate::pagination::OffsetPagination;

pub mod common_bridge;
mod jumbo_bridge;
mod picnic_bridge;

pub(crate) use jumbo_bridge::*;
pub use picnic_bridge::*;

pub trait StaticProviderInfo: ProviderToAny {
    /// The associated [Provider] for this bridge implementation
    fn provider() -> Provider
    where
        Self: Sized;

    /// An object containing all static metadata for this provider.
    fn metadata() -> ProviderMetadata
    where
        Self: Sized;
}

#[async_trait::async_trait]
pub trait ProviderInfo: StaticProviderInfo {
    fn provider(&self) -> Provider;

    fn metadata(&self) -> ProviderMetadata;

    /// If this provider support cart interactions then it will be converted to [ProviderCart].
    fn as_cart_provider(&self) -> Option<&(dyn ProviderCart + Send + Sync)> {
        None
    }

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

/// A trait implemented by a provider if it supports Cart manipulations
///
/// See also [ProviderMetadata::supports_cart]
#[async_trait::async_trait]
pub trait ProviderCart: ProviderInfo {
    /// Add the given item(s) and the quantity thereof to the current cart.
    async fn add_to_cart(&self, items: &[(&ProductIdRef, u32)]) -> Result<()>;

    /// Remove the given item(s) and the quantity thereof from the current cart.
    async fn remove_from_cart(&self, items: &[(&ProductIdRef, u32)]) -> Result<()>;

    /// Clear the current remote cart.
    async fn clear_cart(&self) -> Result<()>;
}

pub trait ProviderToAny: 'static {
    /// Cast self to an [Any](std::any::Any) reference for downcasting.
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: 'static> ProviderToAny for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
