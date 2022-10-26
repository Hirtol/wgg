use crate::{Provider, WggProduct, WggSearchProduct};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;
use std::collections::HashMap;
use std::num::{NonZeroU64, NonZeroUsize};
use std::ops::{Deref, DerefMut};
use std::time::Duration;

type ProductId = String;

#[derive(Serialize, Deserialize)]
pub(crate) struct WggProviderCache {
    full_products: WggCacheMap<WggProduct>,
    search_products: WggCacheMap<WggSearchProduct>,
}

impl WggProviderCache {
    /// Create a new bounded cache instance.
    ///
    /// # Arguments
    /// * `providers` - All providers for whom we wish to create a product cache.
    /// * `max_products` - The maximum amount of products *for each individual provider* after which the oldest item will be evicted.
    /// * `cache_lifetime` - How long the entries should life in the cache before being evicted.
    pub fn new(
        providers: impl Iterator<Item = Provider>,
        max_products: NonZeroUsize,
        cache_lifetime: Duration,
    ) -> Self {
        let mut result = Self {
            full_products: WggCacheMap::new(),
            search_products: WggCacheMap::new(),
        };

        for provider in providers {
            result.full_products.insert(
                provider,
                moka::sync::CacheBuilder::new(max_products.get() as u64)
                    .time_to_live(cache_lifetime)
                    .build(),
            );
            result.search_products.insert(
                provider,
                moka::sync::CacheBuilder::new(max_products.get() as u64)
                    .time_to_live(cache_lifetime)
                    .build(),
            );
        }

        result
    }

    /// Try to find the provided `product_id`.
    ///
    /// If it isn't in the search cache the full product cache will be used instead.
    ///
    /// Taking `&String` as argument is intentional due to the internal cache API being a little stupid.
    pub fn get_search_product(&self, provider: Provider, product_id: &str) -> Option<WggSearchProduct> {
        let search_cache = self.search_products.get(&provider)?;
        if let Some(item) = search_cache.get(product_id) {
            Some(item)
        } else {
            let full_cache = self.full_products.get(&provider)?;

            full_cache.get(product_id).map(|item| item.into())
        }
    }

    pub fn get_product(&self, provider: Provider, product_id: &str) -> Option<WggProduct> {
        let full_cache = self.full_products.get(&provider)?;

        full_cache.get(product_id)
    }

    pub fn insert_search_product(&self, provider: Provider, product: Cow<'_, WggSearchProduct>) -> Option<()> {
        let search_cache = self.search_products.get(&provider)?;
        search_cache.insert(product.id.clone(), product.into_owned());
        Some(())
    }

    pub fn insert_product(&self, provider: Provider, product: Cow<'_, WggProduct>) -> Option<()> {
        let search_cache = self.full_products.get(&provider)?;
        search_cache.insert(product.id.clone(), product.into_owned());
        Some(())
    }
}

#[derive(Clone)]
struct WggCacheMap<I>(HashMap<Provider, moka::sync::Cache<ProductId, I>>);

impl<I> WggCacheMap<I> {
    pub fn new() -> Self {
        WggCacheMap(Default::default())
    }
}

impl<I: Serialize + Clone + Send + Sync + 'static> Serialize for WggCacheMap<I> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sub_serializer = serializer.serialize_map(Some(self.0.len()))?;

        for (provider, cache) in self.0.iter() {
            // Temporary hack to make an easy to write Serializer, allocates a full new HashMap for serialization.
            let p = cache.iter().collect::<HashMap<_, _>>();
            sub_serializer.serialize_entry(provider, &p)?;
        }

        sub_serializer.end()
    }
}

impl<'de, I: Serialize + Deserialize<'de> + Clone + Send + Sync + 'static> WggCacheMap<I> {
    pub fn deserialize_from<D>(deserializer: D, size: NonZeroU64, ttl: Duration) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let nested_map: HashMap<Provider, HashMap<ProductId, I>> = serde::Deserialize::deserialize(deserializer)?;

        let result = nested_map
            .into_iter()
            .map(|(provider, values)| {
                let result = moka::sync::CacheBuilder::new(size.get()).time_to_live(ttl).build();

                for (k, v) in values {
                    result.insert(k, v);
                }

                (provider, result)
            })
            .collect();

        Ok(Self(result))
    }
}

impl<'de, I: Serialize + Deserialize<'de> + Clone + Send + Sync + 'static> Deserialize<'de> for WggCacheMap<I> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::deserialize_from(deserializer, NonZeroU64::new(1000).unwrap(), Duration::from_secs(86400))
    }
}

impl<I> Deref for WggCacheMap<I> {
    type Target = HashMap<Provider, moka::sync::Cache<ProductId, I>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I> DerefMut for WggCacheMap<I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
