use crate::models::WggSaleGroupComplete;
use crate::{Provider, WggProduct, WggSearchProduct};
use chrono::{DateTime, Utc};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::Hash;
use std::num::{NonZeroU64, NonZeroUsize};
use std::ops::{Deref, DerefMut};
use std::time::Duration;

type ProductId = String;

pub struct WggProviderCache {
    full_products: WggCacheMap<WggProduct>,
    search_products: WggCacheMap<WggSearchProduct>,
    promotions: WggCacheMap<WggSaleGroupComplete, String>,
    ttl: chrono::Duration,
}

impl WggProviderCache {
    /// Create a new bounded cache instance.
    ///
    /// # Arguments
    /// * `previous_cache` - Optional previous incarnation of this cache.
    /// * `providers` - All providers for whom we wish to create a product cache.
    /// * `max_products` - The maximum amount of products *for each individual provider* after which the oldest item will be evicted.
    /// * `cache_lifetime` - How long the entries should life in the cache before being evicted.
    pub fn new(
        previous_cache: Option<SerdeWggCache>,
        cache_lifetime: Duration,
        providers: impl Iterator<Item = Provider>,
        max_products: NonZeroUsize,
    ) -> Self {
        let cache = previous_cache.unwrap_or_default();
        let mut result = Self {
            full_products: cache.full_products,
            search_products: cache.search_products,
            promotions: cache.promotions,
            ttl: chrono::Duration::from_std(cache_lifetime).unwrap(),
        };

        for provider in providers {
            result.full_products.entry(provider).or_insert_with(|| {
                moka::sync::CacheBuilder::new(max_products.get() as u64)
                    .time_to_live(cache_lifetime)
                    .build()
            });
            result.search_products.entry(provider).or_insert_with(|| {
                moka::sync::CacheBuilder::new(max_products.get() as u64)
                    .time_to_live(cache_lifetime)
                    .build()
            });
        }

        result
    }

    /// Clones this cache into a new Serializable/Deserializable struct
    pub fn as_serde_cache(&self) -> SerdeWggCache {
        SerdeWggCache {
            full_products: self.full_products.clone(),
            search_products: self.search_products.clone(),
            promotions: self.promotions.clone(),
        }
    }

    /// Turns this cache into a new Serializable/Deserializable struct
    #[allow(dead_code)]
    pub fn into_serde_cache(self) -> SerdeWggCache {
        SerdeWggCache {
            full_products: self.full_products,
            search_products: self.search_products,
            promotions: self.promotions,
        }
    }

    /// Try to find the provided `product_id`.
    ///
    /// If it isn't in the search cache the full product cache will be used instead.
    ///
    /// Taking `&String` as argument is intentional due to the internal cache API being a little stupid.
    pub fn get_search_product(&self, provider: Provider, product_id: &str) -> Option<WggSearchProduct> {
        let search_cache = self.search_products.get(&provider)?;

        if search_cache.contains_key(product_id) {
            self.get_or_invalidate(product_id, &search_cache)
        } else {
            self.get_product(provider, product_id).map(|item| item.into())
        }
    }

    pub fn get_product(&self, provider: Provider, product_id: &str) -> Option<WggProduct> {
        let full_cache = self.full_products.get(&provider)?;

        self.get_or_invalidate(product_id, full_cache)
    }

    pub fn insert_search_product(&self, provider: Provider, product: Cow<'_, WggSearchProduct>) -> Option<()> {
        let search_cache = self.search_products.get(&provider)?;
        let to_insert = CacheEntry {
            entry: product.into_owned(),
            inserted_at: Utc::now(),
        };

        search_cache.insert(to_insert.entry.id.clone(), to_insert);
        Some(())
    }

    pub fn insert_product(&self, provider: Provider, product: Cow<'_, WggProduct>) -> Option<()> {
        let search_cache = self.full_products.get(&provider)?;
        let to_insert = CacheEntry {
            entry: product.into_owned(),
            inserted_at: Utc::now(),
        };

        search_cache.insert(to_insert.entry.id.clone(), to_insert);
        Some(())
    }

    /// Retrieves the associated item with the given key from the given cache.
    ///
    /// If the TTL defined by our application has been exceeded the item will be invalidated and `None` will be returned.
    fn get_or_invalidate<I: Clone + Send + Sync + 'static>(
        &self,
        key: &str,
        cache: &moka::sync::Cache<ProductId, CacheEntry<I>>,
    ) -> Option<I> {
        let entry = cache.get(key)?;

        if let Some(item) = entry.get_if_valid(self.ttl) {
            Some(item)
        } else {
            // Past the time to live
            tracing::trace!(key, "Expiring cache-entry for product with given key");
            cache.invalidate(key);
            None
        }
    }
}

#[derive(Clone)]
pub struct WggCacheMap<I, K = ProductId>(HashMap<Provider, moka::sync::Cache<K, CacheEntry<I>>>)
where
    K: Hash + Eq;

impl<I, K: Hash + Eq> WggCacheMap<I, K> {
    pub fn new() -> Self {
        WggCacheMap(Default::default())
    }
}

impl<I, K> Serialize for WggCacheMap<I, K>
where
    I: Serialize + Clone + Send + Sync + 'static,
    K: Serialize + Clone + Send + Sync + Hash + Eq + 'static,
{
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

impl<'de, I, K> WggCacheMap<I, K>
where
    I: Serialize + Deserialize<'de> + Clone + Send + Sync + 'static,
    K: Serialize + Deserialize<'de> + Clone + Send + Sync + Hash + Eq + 'static,
    HashMap<Provider, moka::sync::Cache<K, CacheEntry<I>>>:
        FromIterator<(Provider, moka::sync::Cache<String, CacheEntry<I>>)>,
{
    pub fn deserialize_from<D>(deserializer: D, size: NonZeroU64, ttl: Duration) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let nested_map: HashMap<Provider, HashMap<ProductId, CacheEntry<I>>> =
            serde::Deserialize::deserialize(deserializer)?;

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

impl<'de, I, K> Deserialize<'de> for WggCacheMap<I, K>
where
    I: Serialize + Deserialize<'de> + Clone + Send + Sync + 'static,
    K: Serialize + Deserialize<'de> + Clone + Send + Sync + Hash + Eq + 'static,
    HashMap<Provider, moka::sync::Cache<K, CacheEntry<I>>>:
        FromIterator<(Provider, moka::sync::Cache<String, CacheEntry<I>>)>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::deserialize_from(deserializer, NonZeroU64::new(1000).unwrap(), Duration::from_secs(86400))
    }
}

impl<I, K: Hash + Eq> Deref for WggCacheMap<I, K> {
    type Target = HashMap<Provider, moka::sync::Cache<K, CacheEntry<I>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I, K: Hash + Eq> DerefMut for WggCacheMap<I, K> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// We keep track of the `inserted_at` time separately from `moka` as there is no way for us to know the TTL.
/// This duplicates the timestamp and conversely increase the memory footprint by 12 bytes. :(
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CacheEntry<I> {
    entry: I,
    inserted_at: DateTime<Utc>,
}

impl<I> CacheEntry<I> {
    /// Return the `entry` value if the entry is still valid given the `ttl`.
    pub fn get_if_valid(self, ttl: chrono::Duration) -> Option<I> {
        if self.inserted_at + ttl > Utc::now() {
            Some(self.entry)
        } else {
            // Past the time to live
            None
        }
    }
}

/// Dummy struct to allow for fine grained serialization
#[derive(Serialize, Deserialize, Clone)]
pub struct SerdeWggCache {
    full_products: WggCacheMap<WggProduct>,
    search_products: WggCacheMap<WggSearchProduct>,
    promotions: WggCacheMap<WggSaleGroupComplete, String>,
}

impl Default for SerdeWggCache {
    fn default() -> Self {
        Self {
            full_products: WggCacheMap::new(),
            search_products: WggCacheMap::new(),
            promotions: WggCacheMap::new(),
        }
    }
}
