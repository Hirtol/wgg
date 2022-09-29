use crate::models::{Provider, WggAutocomplete, WggProduct, WggSaleCategory, WggSearchProduct};
use async_graphql::EnumType;
use std::borrow::Cow;
use std::fmt::Debug;
use std::num::NonZeroUsize;
use std::time::Duration;
use tokio::sync::Mutex;
use wgg_jumbo::BaseJumboApi;
use wgg_picnic::PicnicApi;

use crate::cache::WggProviderCache;
use crate::pagination::OffsetPagination;
pub use error::ProviderError;
use providers::{JumboBridge, PicnicBridge, ProviderInfo};
pub use wgg_picnic::Credentials as PicnicCredentials;

mod cache;
mod error;
pub mod models;
pub mod pagination;
mod providers;

type Result<T> = std::result::Result<T, ProviderError>;

pub struct WggProvider {
    pub(crate) picnic: Option<PicnicBridge>,
    pub(crate) jumbo: JumboBridge,
    cache: Mutex<WggProviderCache>,
}

impl WggProvider {
    /// Create a new collection of providers.
    ///
    /// By default only the `JumboApi` is enabled, see [Self::with_picnic] or [Self::with_picnic_login] to enable `Picnic`.
    pub fn new() -> Self {
        WggProvider {
            picnic: None,
            jumbo: JumboBridge::new(BaseJumboApi::new(Default::default())),
            cache: Mutex::new(WggProviderCache::new(
                Provider::items().iter().map(|i| i.value),
                NonZeroUsize::new(1000).unwrap(),
                Duration::from_secs(86400),
            )),
        }
    }

    /// Create a new provider from pre-existing *valid* [wgg_picnic::Credentials].
    pub fn with_picnic(mut self, picnic_credentials: PicnicCredentials) -> Self {
        self.picnic = PicnicBridge::new(PicnicApi::new(picnic_credentials, Default::default())).into();

        self
    }

    /// Initialise the Picnic API provider.
    ///
    /// Ideally one would persist the acquired credentials to disk, and in the future use [Self::with_picnic].
    pub async fn with_picnic_login(mut self, username: &str, password: &str) -> Result<Self> {
        let picnic = PicnicApi::from_login(username, password, Default::default()).await?;

        self.picnic = Some(PicnicBridge::new(picnic));

        Ok(self)
    }

    /// Return the currently used [PicnicCredentials], if the Picnic provider has been created.
    ///
    /// If not, [None] is returned.
    pub fn picnic_credentials(&self) -> Option<&PicnicCredentials> {
        Some(self.picnic.as_ref()?.api.credentials())
    }

    /// Provide autocomplete results from the requested [Provider].
    ///
    /// Note that for some providers it is *very* important to use their returned suggestions, or else the [Self::search] will perform poorly
    #[tracing::instrument(level="debug", skip_all, fields(query = query.as_ref()))]
    pub async fn autocomplete(&self, provider: Provider, query: impl AsRef<str>) -> Result<Vec<WggAutocomplete>> {
        #[cached::proc_macro::cached(
            size = 100,
            time = 86400,
            result = true,
            key = "String",
            convert = r#"{query.to_string()}"#
        )]
        async fn inner(prov: &(dyn ProviderInfo + Send + Sync), query: &str) -> Result<Vec<WggAutocomplete>> {
            prov.autocomplete(query).await
        }

        let provider = self.find_provider(provider)?;

        inner(provider, query.as_ref()).await
    }

    /// Search for the provided query in the given [Provider].
    ///
    /// For searching all providers at the same time see [Self::search_all]
    #[tracing::instrument(level="debug", skip_all, fields(query = query.as_ref()))]
    pub async fn search(
        &self,
        provider: Provider,
        query: impl AsRef<str>,
        offset: Option<u32>,
    ) -> Result<OffsetPagination<WggSearchProduct>> {
        #[cached::proc_macro::cached(
            size = 100,
            time = 86400,
            result = true,
            key = "(String, Option<u32>)",
            convert = r#"{(query.to_string(), offset)}"#
        )]
        async fn inner(
            prov: &(dyn ProviderInfo + Send + Sync),
            query: &str,
            offset: Option<u32>,
        ) -> Result<OffsetPagination<WggSearchProduct>> {
            prov.search(query, offset).await
        }

        let provider_concrete = self.find_provider(provider)?;

        let result = inner(provider_concrete, query.as_ref(), offset).await?;

        // We persist any and all products for the sake of easing custom list searches.
        let mut guard = self.cache.lock().await;

        for item in &result.items {
            guard.insert_search_product(provider, Cow::Borrowed(item));
        }

        Ok(result)
    }

    /// Search all providers for the given query.
    ///
    /// The [OffsetPagination] will have no `offset` listed, but the `total_items` will be the sum of all APIs' total items.
    #[tracing::instrument(level="debug", skip_all, fields(query = query.as_ref()))]
    pub async fn search_all(&self, query: impl AsRef<str>) -> Result<OffsetPagination<WggSearchProduct>> {
        #[cached::proc_macro::cached(
            size = 100,
            time = 86400,
            result = true,
            key = "String",
            convert = r#"{query.to_string()}"#
        )]
        async fn inner(
            prov: &(dyn ProviderInfo + Send + Sync),
            query: &str,
        ) -> Result<OffsetPagination<WggSearchProduct>> {
            prov.search(query, None).await
        }

        let queries = self.iter().map(|i| inner(i, query.as_ref()));

        let results = futures::future::join_all(queries)
            .await
            .into_iter()
            .flatten()
            .reduce(|mut accum, mut item| {
                accum.items.append(&mut item.items);
                accum.total_items += item.total_items;
                accum
            })
            .ok_or(ProviderError::NothingFound)?;

        // We persist any and all products for the sake of easing custom list searches.
        let mut guard = self.cache.lock().await;

        for item in &results.items {
            guard.insert_search_product(item.provider, Cow::Borrowed(item));
        }

        Ok(results)
    }

    /// Retrieve all valid promotions for the current week for the given provider.
    #[tracing::instrument(level = "debug", skip_all, fields(provider))]
    pub async fn promotions(&self, provider: Provider) -> Result<Vec<WggSaleCategory>> {
        #[cached::proc_macro::cached(
            size = 100,
            time = 86400,
            result = true,
            key = "Provider",
            convert = r#"{_provider}"#
        )]
        async fn inner(prov: &(dyn ProviderInfo + Send + Sync), _provider: Provider) -> Result<Vec<WggSaleCategory>> {
            prov.promotions().await
        }

        let prov = self.find_provider(provider)?;

        inner(prov, provider).await
    }

    /// Retrieve all valid promotions for the current week.
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn promotions_all(&self) -> Result<Vec<WggSaleCategory>> {
        let provider = self.iter().map(|i| i.promotions());

        futures::future::join_all(provider)
            .await
            .into_iter()
            .flatten()
            .reduce(|mut accum, mut item| {
                accum.append(&mut item);
                accum
            })
            .ok_or(ProviderError::NothingFound)
    }

    /// Retrieve all products that are part of the given promotion sub-list.
    #[tracing::instrument(level = "debug", skip_all, fields(provider, sublist_id))]
    pub async fn promotions_sublist(
        &self,
        provider: Provider,
        sublist_id: impl AsRef<str>,
    ) -> Result<OffsetPagination<WggSearchProduct>> {
        #[cached::proc_macro::cached(
            size = 100,
            time = 86400,
            result = true,
            key = "String",
            convert = r#"{sublist_id.to_string()}"#
        )]
        async fn inner(
            prov: &(dyn ProviderInfo + Send + Sync),
            sublist_id: &str,
        ) -> Result<OffsetPagination<WggSearchProduct>> {
            prov.promotions_sublist(sublist_id).await
        }

        let provider = self.find_provider(provider)?;

        let results = inner(provider, sublist_id.as_ref()).await?;

        // We persist any and all products for the sake of easing custom list searches.
        let mut guard = self.cache.lock().await;

        for item in &results.items {
            guard.insert_search_product(item.provider, Cow::Borrowed(item));
        }

        Ok(results)
    }

    /// Retrieve the provided `product_id` from the `provider`.
    ///
    /// Note that this `product_id` needs to be obtained from this specific `provider`. Product ids do not cross provider boundaries.
    #[tracing::instrument(level="debug", skip_all, fields(provider, query = product_id.as_ref()))]
    pub async fn product(&self, provider: Provider, product_id: impl AsRef<str>) -> Result<WggProduct> {
        let mut guard = self.cache.lock().await;

        self.product_cache_or_network(provider, &product_id.as_ref().to_string(), &mut guard)
            .await
    }

    /// Retrieve the search product representation of the requested product.
    ///
    /// This is highly recommended for the majority of cases to reduce latency and external network calls as several
    /// cache layers can be used at once.
    #[tracing::instrument(level="debug", skip_all, fields(provider, query = product_id.as_ref()))]
    pub async fn search_product_by_id(
        &self,
        provider: Provider,
        product_id: impl AsRef<str>,
    ) -> Result<WggSearchProduct> {
        let id = product_id.as_ref().to_string();
        let mut guard = self.cache.lock().await;
        if let Some(item) = guard.get_search_product(provider, &id) {
            Ok(item)
        } else {
            Ok(self.product_cache_or_network(provider, &id, &mut guard).await?.into())
        }
    }

    /// Retrieve the search product representation of the requested product.
    ///
    /// This is highly recommended for the majority of cases to reduce latency and external network calls as several
    /// cache layers can be used at once.
    ///
    /// Calling this is more efficient than individual [Self::search_product_by_id] calls for multiple ids as it allows
    /// keeping the Mutex guard.
    #[tracing::instrument(level = "debug", skip_all, fields(provider))]
    pub async fn search_products_by_id(
        &self,
        provider: Provider,
        product_ids: impl IntoIterator<Item = impl AsRef<str> + Debug>,
    ) -> Result<Vec<WggSearchProduct>> {
        let mut guard = self.cache.lock().await;
        let mut result = Vec::with_capacity(2);

        for product in product_ids {
            let id = product.as_ref().to_string();
            if let Some(item) = guard.get_search_product(provider, &id) {
                result.push(item);
            } else {
                result.push(self.product_cache_or_network(provider, &id, &mut guard).await?.into());
            }
        }

        Ok(result)
    }

    /// Perform a cache lookup and, upon a cache miss, request the current product.
    ///
    /// Takes a mutable reference to the cache to allow this to be called in a loop, without re-acquiring the Mutex
    /// every time.
    async fn product_cache_or_network(
        &self,
        provider: Provider,
        product_id: &String,
        cache: &mut WggProviderCache,
    ) -> Result<WggProduct> {
        if let Some(item) = cache.get_product(provider, product_id) {
            Ok(item)
        } else {
            let provider_concrete = self.find_provider(provider)?;
            let result = provider_concrete.product(product_id).await?;

            cache.insert_product(provider, Cow::Borrowed(&result));

            Ok(result)
        }
    }

    /// Return a reference to the requested provider.
    fn find_provider(&self, provider: Provider) -> Result<&(dyn ProviderInfo + Send + Sync)> {
        match provider {
            Provider::Picnic => self
                .picnic
                .as_ref()
                .map(|p| p as &(dyn ProviderInfo + Send + Sync))
                .ok_or(ProviderError::ProviderUninitialised(Provider::Picnic)),
            Provider::Jumbo => Ok(&self.jumbo),
        }
    }

    /// Iterate over all providers allowing an action to be performed on all of them
    fn iter(&self) -> ProvidersIter<'_> {
        ProvidersIter { providers: self, i: 0 }
    }
}

impl Default for WggProvider {
    fn default() -> Self {
        Self::new()
    }
}

struct ProvidersIter<'a> {
    providers: &'a WggProvider,
    i: usize,
}

impl<'a> Iterator for ProvidersIter<'a> {
    type Item = &'a (dyn ProviderInfo + Send + Sync);

    fn next(&mut self) -> Option<Self::Item> {
        let result: Option<&(dyn ProviderInfo + Send + Sync)> = match self.i {
            0 => {
                let provider = self.providers.find_provider(Provider::Picnic);

                if provider.is_err() {
                    self.i += 1;
                    self.next()
                } else {
                    provider.ok()
                }
            }
            1 => Some(&self.providers.jumbo),
            _ => None,
        };

        self.i += 1;

        result
    }
}
