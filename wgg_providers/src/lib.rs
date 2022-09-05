use crate::models::{Autocomplete, Product, Provider, SearchProduct};
use wgg_jumbo::BaseJumboApi;
use wgg_picnic::PicnicApi;

use crate::pagination::OffsetPagination;
pub use error::ProviderError;
use providers::{JumboBridge, PicnicBridge, ProviderInfo};
pub use wgg_picnic::Credentials as PicnicCredentials;

mod error;
pub mod models;
pub mod pagination;
mod providers;

type Result<T> = std::result::Result<T, ProviderError>;

pub struct WggProvider {
    pub(crate) picnic: Option<PicnicBridge>,
    pub(crate) jumbo: JumboBridge,
}

impl WggProvider {
    /// Create a new collection of providers.
    ///
    /// By default only the `JumboApi` is enabled, see [Self::with_picnic] or [Self::with_picnic_login] to enable `Picnic`.
    pub fn new() -> Self {
        WggProvider {
            picnic: None,
            jumbo: JumboBridge::new(BaseJumboApi::new(Default::default())),
        }
    }

    /// Create a new provider from pre-existing *valid* [wgg_picnic::Credentials].
    pub fn with_picnic(mut self, picnic_credentials: PicnicCredentials) -> Self {
        self.picnic = PicnicBridge::new(PicnicApi::new(picnic_credentials, Default::default())).into();

        self
    }

    /// Initialise the Picnic API provider.
    ///
    /// Ideally one would persist the acquired credentials to disk.
    pub async fn with_picnic_login(mut self, username: &str, password: &str) -> Result<Self> {
        let picnic = PicnicApi::from_login(username, password, Default::default()).await?;

        self.picnic = Some(PicnicBridge::new(picnic));

        Ok(self)
    }

    /// Provide autocomplete results from the requested [Provider].
    ///
    /// Note that for some providers it is *very* important to use their returned suggestions, or else the [Self::search] will perform poorly
    #[tracing::instrument(level="debug", skip_all, fields(query = query.as_ref()))]
    pub async fn autocomplete(&self, provider: Provider, query: impl AsRef<str>) -> Result<Vec<Autocomplete>> {
        let provider = self.find_provider(provider)?;

        provider.autocomplete(query.as_ref()).await
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
    ) -> Result<OffsetPagination<SearchProduct>> {
        let provider = self.find_provider(provider)?;

        provider.search(query.as_ref(), offset).await
    }

    /// Search all providers for the given query.
    ///
    /// The [OffsetPagination] will have no `offset` listed, but the `total_items` will be the sum of all APIs' total items.
    #[tracing::instrument(level="debug", skip_all, fields(query = query.as_ref()))]
    pub async fn search_all(&self, query: impl AsRef<str>) -> Result<OffsetPagination<SearchProduct>> {
        let provider = self.iter().map(|i| i.search(query.as_ref(), None));

        futures::future::join_all(provider)
            .await
            .into_iter()
            .flatten()
            .reduce(|mut accum, mut item| {
                accum.items.append(&mut item.items);
                accum.total_items += item.total_items;
                accum
            })
            .ok_or(ProviderError::NothingFound)
    }

    /// Retrieve the provided `product_id` from the `provider`.
    ///
    /// Note that this `product_id` needs to be obtained from this specific `provider`. Product ids do not cross provider boundaries.
    #[tracing::instrument(level="debug", skip_all, fields(provider, query = product_id.as_ref()))]
    pub async fn product(&self, provider: Provider, product_id: impl AsRef<str>) -> Result<Product> {
        let provider = self.find_provider(provider)?;

        provider.product(product_id.as_ref()).await
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
