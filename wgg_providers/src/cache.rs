use crate::{Provider, WggProduct, WggSearchProduct};
use cached::Cached;
use lru::LruCache;
use std::borrow::Cow;
use std::num::NonZeroUsize;
use std::time::Duration;

type ProductId = String;

pub(crate) struct WggProviderCache {
    pub full_product: LruCache<Provider, cached::TimedSizedCache<ProductId, WggProduct>>,
    pub search_product: LruCache<Provider, cached::TimedSizedCache<ProductId, WggSearchProduct>>,
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
            full_product: LruCache::unbounded(),
            search_product: LruCache::unbounded(),
        };

        for provider in providers {
            result.full_product.push(
                provider,
                cached::TimedSizedCache::with_size_and_lifespan(max_products.get(), cache_lifetime.as_secs()),
            );
            result.search_product.push(
                provider,
                cached::TimedSizedCache::with_size_and_lifespan(max_products.get(), cache_lifetime.as_secs()),
            );
        }

        result
    }

    /// Try to find the provided `product_id`.
    ///
    /// If it isn't in the search cache the full product cache will be used instead.
    ///
    /// Taking `&String` as argument is intentional due to the internal cache API being a little stupid.
    pub fn get_search_product(&mut self, provider: Provider, product_id: &String) -> Option<WggSearchProduct> {
        let search_cache = self.search_product.get_mut(&provider)?;
        if let Some(item) = search_cache.cache_get(product_id) {
            Some(item.clone())
        } else {
            let full_cache = self.full_product.get_mut(&provider)?;

            full_cache.cache_get(product_id).map(|item| item.clone().into())
        }
    }

    pub fn get_product(&mut self, provider: Provider, product_id: &String) -> Option<WggProduct> {
        let full_cache = self.full_product.get_mut(&provider)?;

        full_cache.cache_get(product_id).cloned()
    }

    pub fn insert_search_product(&mut self, provider: Provider, product: Cow<'_, WggSearchProduct>) -> Option<()> {
        let search_cache = self.search_product.get_mut(&provider)?;
        search_cache.cache_set(product.id.clone(), product.into_owned());
        Some(())
    }

    pub fn insert_product(&mut self, provider: Provider, product: Cow<'_, WggProduct>) -> Option<()> {
        let search_cache = self.full_product.get_mut(&provider)?;
        search_cache.cache_set(product.id.clone(), product.into_owned());
        Some(())
    }
}
