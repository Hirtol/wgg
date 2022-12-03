use crate::models::{
    Provider, WggAutocomplete, WggProduct, WggSaleCategory, WggSaleGroupComplete, WggSaleItem, WggSearchProduct,
};
use async_graphql::EnumType;
use std::fmt::Debug;
use std::num::{NonZeroU32, NonZeroUsize};
use std::sync::Arc;
use std::time::Duration;
use wgg_jumbo::BaseJumboApi;

use crate::caching::SerdeCache;
use crate::caching::WggProviderCache;
use crate::error::ProviderError;
use crate::pagination::OffsetPagination;
use crate::providers::PicnicCredentials;
use crate::providers::{JumboBridge, PicnicBridge, ProviderInfo};
use crate::sale_resolver::{SaleInfo, SaleResolver};
use crate::Result;
use wgg_scheduler::JobScheduler;

pub struct WggProvider {
    pub(crate) picnic: Option<PicnicBridge>,
    pub(crate) jumbo: JumboBridge,
    pub(crate) cache: WggProviderCache,
    pub(crate) sales: SaleResolver,
}

impl WggProvider {
    /// Start creating a new collection of providers
    pub fn builder() -> WggProviderBuilder {
        WggProviderBuilder::new()
    }

    /// Return a serializable form of the in-memory product cache used for quick responses within the `WggProvider` instance.
    pub fn serialized_cache(&self) -> SerdeCache {
        SerdeCache {
            product_cache: self.cache.as_serde_cache(),
            promotions_cache: self.sales.cache().clone(),
        }
    }

    /// Returns the latest Picnic auth token in use, if the provider has been initialised with [with_picnic](WggProviderBuilder::with_picnic)
    /// in the builder.
    pub async fn picnic_auth_token(&self) -> Option<String> {
        let picnic = self.picnic.as_ref()?;
        Some(picnic.credentials().await.auth_token)
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
    /// `offset` will always be respected, even if the underlying API does not support it.
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
            key = "(String, Option<u32>, Provider)",
            convert = r#"{(query.to_string(), offset, _provider)}"#
        )]
        async fn inner(
            prov: &(dyn ProviderInfo + Send + Sync),
            query: &str,
            offset: Option<u32>,
            _provider: Provider,
        ) -> Result<OffsetPagination<WggSearchProduct>> {
            prov.search(query, offset).await
        }

        let provider_concrete = self.find_provider(provider)?;

        let result = inner(provider_concrete, query.as_ref(), offset, provider_concrete.provider()).await?;

        // We persist any and all products for the sake of easing custom list searches.

        for item in &result.items {
            self.cache.insert_search_product(provider, item.clone());
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
            key = "(String, Provider)",
            convert = r#"{(query.to_string(), _provider)}"#
        )]
        async fn inner(
            prov: &(dyn ProviderInfo + Send + Sync),
            query: &str,
            _provider: Provider,
        ) -> Result<OffsetPagination<WggSearchProduct>> {
            prov.search(query, None).await
        }

        let queries = self.active_providers().map(|i| inner(i, query.as_ref(), i.provider()));

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
        for item in &results.items {
            self.cache.insert_search_product(item.provider, item.clone());
        }

        Ok(results)
    }

    /// Retrieve all valid promotions for the current week for the given provider.
    #[tracing::instrument(level = "debug", skip_all, fields(provider))]
    pub async fn promotions(&self, provider: Provider) -> Result<Vec<WggSaleCategory>> {
        let prov = self.find_provider(provider)?;

        if let Some(promos) = self.sales.promotions(provider, &self).await {
            Ok(promos)
        } else {
            let result = prov.promotions().await?;

            self.sales.insert_promotions(provider, result.clone(), self).await;

            // Persist any extra search products as we find them
            for category in &result {
                for item in &category.items {
                    if let WggSaleItem::Product(product) = item {
                        self.cache.insert_search_product(provider, product.clone());
                    }
                }
            }

            Ok(result)
        }
    }

    /// Retrieve all valid promotions for the current week.
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn promotions_all(&self) -> Result<Vec<WggSaleCategory>> {
        let provider = self.active_providers().map(|i| self.promotions(i.provider()));

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
    ) -> Result<WggSaleGroupComplete> {
        if let Some(result) = self.sales.promotion_sublist(provider, sublist_id.as_ref(), &self).await {
            Ok(result)
        } else {
            let prov = self.find_provider(provider)?;
            let result = prov.promotions_sublist(sublist_id.as_ref()).await?;

            let _ = self
                .sales
                .insert_promotion_sublist(provider, result.clone(), self)
                .await;

            // We persist any and all products for the sake of easing custom list searches.
            for item in &result.items {
                self.cache.insert_search_product(item.provider, item.clone());
            }

            Ok(result)
        }
    }

    /// Retrieve the provided `product_id` from the `provider`.
    ///
    /// Note that this `product_id` needs to be obtained from this specific `provider`. Product ids do not cross provider boundaries.
    #[tracing::instrument(level="debug", skip_all, fields(provider, query = product_id.as_ref()))]
    pub async fn product(&self, provider: Provider, product_id: impl AsRef<str>) -> Result<WggProduct> {
        if let Some(item) = self.cache.get_product(provider, product_id.as_ref()) {
            Ok(item)
        } else {
            self.product_network(provider, product_id.as_ref()).await
        }
    }

    /// Retrieve the search product representation of the requested product.
    ///
    /// This is highly recommended for the majority of cases to reduce latency and external network calls as several
    /// cache layers can be used at once.
    #[tracing::instrument(level="debug", skip_all, fields(provider, product_id = product_id.as_ref()))]
    pub async fn search_product(&self, provider: Provider, product_id: impl AsRef<str>) -> Result<WggSearchProduct> {
        let id = product_id.as_ref();

        if let Some(item) = self.cache.get_search_product(provider, id) {
            Ok(item)
        } else {
            Ok(self.product_network(provider, id).await?.into())
        }
    }

    /// Retrieve the search product representation of the requested product.
    ///
    /// Calling this allows for easy concurrency of requests as opposed to individual [search_product](Self::search_product) calls.
    #[tracing::instrument(level = "debug", skip_all, fields(provider))]
    pub async fn search_products(
        &self,
        provider: Provider,
        product_ids: impl IntoIterator<Item = impl AsRef<str> + Debug>,
    ) -> Result<Vec<WggSearchProduct>> {
        use futures::stream::{StreamExt, TryStreamExt};

        futures::stream::iter(product_ids)
            .map(|id| async move {
                let id = id.as_ref();

                if let Some(item) = self.cache.get_search_product(provider, id) {
                    Ok(item)
                } else {
                    self.product_network(provider, id).await.map(|i| i.into())
                }
            })
            .buffer_unordered(10)
            .try_collect()
            .await
    }

    /// Retrieve the associated sale for this item
    pub async fn product_sale_association(&self, provider: Provider, product_id: impl AsRef<str>) -> Result<SaleInfo> {
        self.sales
            .get_sale_info(provider, product_id.as_ref())
            .ok_or(ProviderError::NothingFound)
    }

    /// Push all jobs relevant for optimal service of [WggProvider]s onto the given scheduler
    ///
    /// These services are *not* mandatory for this to work, but they *are* mandatory for things to stay up-to-date.
    pub fn schedule_all_jobs(self: Arc<Self>, scheduler: &JobScheduler) {
        crate::scheduled_jobs::schedule_all_jobs(scheduler, self)
    }

    /// Perform a network request for the requested product.
    async fn product_network(&self, provider: Provider, product_id: &str) -> Result<WggProduct> {
        let provider_concrete = self.find_provider(provider)?;
        let result = provider_concrete.product(product_id).await?;

        self.cache.insert_product(provider, result.clone());

        Ok(result)
    }

    /// Return a reference to the requested provider.
    pub(crate) fn find_provider(&self, provider: Provider) -> Result<&(dyn ProviderInfo + Send + Sync)> {
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
    pub fn active_providers(&self) -> ProvidersIter<'_> {
        ProvidersIter { providers: self, i: 0 }
    }
}

pub struct ProvidersIter<'a> {
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

#[derive(Default)]
pub struct WggProviderBuilder {
    picnic_creds: Option<PicnicCredentials>,
    picnic_rps: Option<NonZeroU32>,
    jumbo: Option<BaseJumboApi>,
    cache: Option<SerdeCache>,
    startup_validation: bool,
}

impl WggProviderBuilder {
    /// Create a new [WggProviderBuilder] instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialise the Picnic API provider.
    ///
    /// Should one provide `auth_token` to the [PicnicCredentials] then the initial login attempt is skipped and this
    /// future resolves immediately.
    pub fn with_picnic(mut self, picnic_credentials: PicnicCredentials) -> Self {
        self.picnic_creds = Some(picnic_credentials);
        self
    }

    /// Provide a non-default RPS limit for the `Picnic` service.
    ///
    /// If this isn't called the default [recommended rps](providers::PICNIC_RECOMMENDED_RPS) is used instead.
    pub fn with_picnic_rps(mut self, limit_rps: Option<NonZeroU32>) -> Self {
        self.picnic_rps = limit_rps;
        self
    }

    /// Provide a non-standard Jumbo config.
    ///
    /// Even if this is not called the Jumbo service is still available.
    pub fn with_jumbo(mut self, config: wgg_jumbo::Config) -> Self {
        self.jumbo = Some(BaseJumboApi::new(config));
        self
    }

    /// Provide a persistent cache, to be called back before the end of the program.
    pub fn with_cache(mut self, cache: Option<SerdeCache>) -> Self {
        self.cache = cache;
        self
    }

    /// Whether to launch an asynchronous fetching of sale/promotion data as soon as the [WggProvider] is constructed.
    pub fn with_startup_sale_validation(mut self, startup_validation: bool) -> Self {
        self.startup_validation = startup_validation;
        self
    }

    /// Create a new collection of providers.
    ///
    /// By default only the `JumboApi` is enabled, see [Self::with_picnic] to enable `Picnic`.
    #[tracing::instrument(level = "info", skip_all)]
    pub async fn build(self) -> Result<WggProvider> {
        let providers = Provider::items().iter().map(|i| i.value);
        let (sales_cache, product_cache) = if let Some(cache) = self.cache {
            (Some(cache.promotions_cache), Some(cache.product_cache))
        } else {
            (None, None)
        };

        let product_cache = WggProviderCache::new(
            product_cache,
            Duration::from_secs(86400),
            providers.clone(),
            NonZeroUsize::new(1000).unwrap(),
        );
        let sales = SaleResolver::new(providers, sales_cache);

        let jumbo = self.jumbo.unwrap_or_else(|| BaseJumboApi::new(Default::default()));
        let picnic = if let Some(credentials) = self.picnic_creds {
            let rps = self.picnic_rps.or(crate::providers::PICNIC_RECOMMENDED_RPS);
            Some(PicnicBridge::new(credentials, rps).await?)
        } else {
            None
        };

        let result = WggProvider {
            picnic,
            jumbo: JumboBridge::new(jumbo),
            cache: product_cache,
            sales,
        };

        if self.startup_validation {
            tracing::debug!("Starting start-up sale validation");
            let futures = result
                .active_providers()
                .map(|provider| result.sales.refresh_promotions(provider.provider(), &result));
            let _ = futures::future::join_all(futures).await;
        }

        Ok(result)
    }
}
