use crate::caching::product_cache::SerdeWggCache;
use serde::{Deserialize, Serialize};

mod product_cache;

use crate::models::Provider;
use crate::sale_resolver::PromotionsCache;
pub use product_cache::WggProviderCache;

#[derive(Serialize, Deserialize, Clone)]
pub struct SerdeCache {
    pub(crate) product_cache: SerdeWggCache,
    pub(crate) promotions_cache: PromotionsCache,
}

pub(crate) fn get_default_provider_map<T: Default, B: FromIterator<(Provider, T)>>(
    providers: impl Iterator<Item = Provider>,
) -> B {
    providers.map(|i| (i, T::default())).collect()
}
