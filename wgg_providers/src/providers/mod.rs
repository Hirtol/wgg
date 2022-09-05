use crate::{Autocomplete, OffsetPagination, Provider, SearchProduct};

mod common_bridge;
mod jumbo_bridge;
mod picnic_bridge;

pub(crate) use jumbo_bridge::*;
pub(crate) use picnic_bridge::*;

#[async_trait::async_trait]
pub trait ProviderInfo {
    fn provider() -> Provider
    where
        Self: Sized;

    /// Perform an autocomplete match for the provided query.
    ///
    /// Some APIs will perform a network call, whilst others will do in-process filtering to provide a list of terms.
    async fn autocomplete(&self, query: &str) -> crate::Result<Vec<Autocomplete>>;

    /// Perform a search for the provided term, and return the pages.
    ///
    /// # Arguments
    /// * `offset` - Can be `None` for APIs which don't support pagination, but should be used for others which do.
    async fn search(&self, query: &str, offset: Option<u32>) -> crate::Result<OffsetPagination<SearchProduct>>;
}
