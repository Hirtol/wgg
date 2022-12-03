use crate::models::Provider;
pub use crate::providers::PicnicCredentials;
use crate::providers::ProviderInfo;
pub use caching::SerdeCache;
pub use error::ProviderError;
pub use providers::PICNIC_RECOMMENDED_RPS;
pub use sale_resolver::SaleInfo;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
pub use wgg_provider::{ProvidersIter, WggProvider, WggProviderBuilder};

mod caching;
mod error;
pub mod models;
pub mod pagination;
mod providers;
mod sale_resolver;
mod scheduled_jobs;
mod wgg_provider;

pub(crate) type ProviderMap<T> = HashMap<Provider, T>;
pub(crate) type DynProvider = dyn ProviderInfo + Send + Sync;

#[derive(Clone, Default)]
pub(crate) struct DynamicProviders(ProviderMap<Arc<DynProvider>>);

impl DynamicProviders {
    pub fn new() -> Self {
        Self(Default::default())
    }

    /// Return a reference to the requested provider.
    pub(crate) fn find_provider(&self, provider: Provider) -> error::Result<&DynProvider> {
        self.0
            .get(&provider)
            .ok_or(ProviderError::ProviderUninitialised(provider))
            .map(|i| i.deref())
    }
}

impl Deref for DynamicProviders {
    type Target = ProviderMap<Arc<DynProvider>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DynamicProviders {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
