use crate::caching::product_cache::SerdeWggCache;
use serde::{Deserialize, Serialize};

mod product_cache;
mod sale_resolution;

pub use product_cache::WggProviderCache;
pub use sale_resolution::{scheduled, SaleInfo, SaleResolver};

#[derive(Serialize, Deserialize, Clone)]
pub struct SerdeCache {
    pub(crate) product_cache: SerdeWggCache,
    pub(crate) promotions_cache: SaleResolver,
}
