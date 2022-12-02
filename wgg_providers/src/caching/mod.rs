use crate::caching::product_cache::SerdeWggCache;
use serde::{Deserialize, Serialize};

mod product_cache;
mod promotions_cache;

use crate::models::Provider;
pub use crate::sale_resolver::scheduled;
pub use crate::sale_resolver::SaleInfo;
pub use crate::sale_resolver::SaleResolver;
pub use product_cache::WggProviderCache;
pub use promotions_cache::PromotionsCache;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct SerdeCache {
    pub(crate) product_cache: SerdeWggCache,
    pub(crate) promotions_cache: PromotionsCache,
}

pub type ProviderMap<T> = HashMap<Provider, T>;

pub(crate) fn get_default_provider_map<T: Default, B: FromIterator<(Provider, T)>>(
    providers: impl Iterator<Item = Provider>,
) -> B {
    providers.map(|i| (i, T::default())).collect()
}
