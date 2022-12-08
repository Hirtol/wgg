use crate::caching::get_default_provider_map;
use crate::models::{ProductId, Provider, SublistId, WggSaleCategory, WggSaleGroupComplete};
use crate::{ProviderMap, SaleInfo};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PromotionsCache {
    promotions_cache: DashMap<Provider, PromoCacheEntry<Vec<WggSaleCategory>>>,
    sublist_cache: ProviderMap<DashMap<SublistId, PromoCacheEntry<WggSaleGroupComplete>>>,
    normal_cache: ProviderMap<DashMap<SublistId, SaleInfo>>,
    inverted_cache: ProviderMap<DashMap<ProductId, SublistId>>,
}

impl PromotionsCache {
    pub fn new(providers: impl Iterator<Item = Provider> + Clone) -> Self {
        Self {
            promotions_cache: get_default_provider_map(providers.clone()),
            sublist_cache: get_default_provider_map(providers.clone()),
            normal_cache: get_default_provider_map(providers.clone()),
            inverted_cache: get_default_provider_map(providers),
        }
    }

    /// To be called after restoring the cache from disk.
    ///
    /// The server might have switched off/on a new provider in that time, and we need to make space for it while we still
    /// can!
    pub(crate) fn restore_from_cached_state(&mut self, active_providers: impl Iterator<Item = Provider> + Clone) {
        let promotions_cache = self.promotions_cache.iter().map(|k| *k.key()).collect::<HashSet<_>>();
        let other = active_providers.collect::<HashSet<_>>();
        let diff = promotions_cache.symmetric_difference(&other);

        for difference in diff {
            // We no longer have the given provider in our active list.
            if promotions_cache.contains(difference) {
                self.promotions_cache.remove(difference);
                self.sublist_cache.remove(difference);
                self.normal_cache.remove(difference);
                self.inverted_cache.remove(difference);
            } else {
                // This provider is new! We had better make space for it.
                self.promotions_cache.insert(*difference, Default::default());
                self.sublist_cache.insert(*difference, Default::default());
                self.normal_cache.insert(*difference, Default::default());
                self.inverted_cache.insert(*difference, Default::default());
            }
        }
    }

    pub(super) fn derived_caches(&self, provider: Provider) -> Option<DerivedCaches<'_>> {
        Some(DerivedCaches {
            normal_cache: self.normal_cache.get(&provider)?,
            inverted_cache: self.inverted_cache.get(&provider)?,
        })
    }

    pub(super) fn promotions(&self, provider: Provider) -> Result<Vec<WggSaleCategory>, CacheAction> {
        let item = self.promotions_cache.get(&provider).ok_or(CacheAction::Nothing)?;

        if item.is_expired() {
            drop(item);
            let _ = self.promotions_cache.remove(&provider);
            self.clear_all_derived(provider).ok_or(CacheAction::Nothing)?;
            Err(CacheAction::ReconcileCache)
        } else {
            Ok(item.item.clone())
        }
    }

    pub(super) fn promotion_sublist(
        &self,
        provider: Provider,
        sublist_id: &str,
    ) -> Result<WggSaleGroupComplete, CacheAction> {
        let sublist_cache = self.sublist_cache.get(&provider).ok_or(CacheAction::Nothing)?;
        let item = sublist_cache.get(sublist_id).ok_or(CacheAction::Nothing)?;

        if item.is_expired() {
            drop(item);
            let _ = sublist_cache.remove(sublist_id);
            self.clear_derived_sublist(provider, sublist_id);
            Err(CacheAction::ReconcileCache)
        } else {
            Ok(item.item.clone())
        }
    }

    pub(super) fn insert_promotions(&self, provider: Provider, promos: Vec<WggSaleCategory>) -> CacheAction {
        let now = Utc::now();
        let cache = PromoCacheEntry {
            item: promos,
            inserted_at: now,
            expires: crate::providers::common_bridge::get_guessed_sale_validity(now).valid_until,
        };

        let _ = self.promotions_cache.insert(provider, cache);

        CacheAction::ReconcileCache
    }

    pub(super) fn insert_promotion_sublist(
        &self,
        provider: Provider,
        promo: WggSaleGroupComplete,
    ) -> Option<CacheAction> {
        let promo_id = promo.id.clone();

        let cache = PromoCacheEntry {
            inserted_at: Utc::now(),
            expires: promo.sale_info.sale_validity.valid_until,
            item: promo,
        };

        let provider = self.sublist_cache.get(&provider)?;
        let old_item = provider.insert(promo_id, cache);

        if old_item.is_some() {
            Some(CacheAction::ReconcileCache)
        } else {
            Some(CacheAction::Nothing)
        }
    }

    /// Pre-emptively clear all currently expired entries.
    pub(crate) fn clear_expired(&self) {
        self.promotions_cache.retain(|provider, value| {
            if value.is_expired() {
                let _ = self.clear_all_derived(*provider);
            }

            !value.is_expired()
        });

        for (_, dash) in self.sublist_cache.iter() {
            dash.retain(|sublist_id, value| {
                if value.is_expired() {
                    self.clear_derived_sublist(value.provider, sublist_id);
                }

                !value.is_expired()
            });
        }
    }

    pub(super) fn clear_all_derived(&self, provider: Provider) -> Option<()> {
        let derived = self.derived_caches(provider)?;
        derived.normal_cache.clear();
        derived.inverted_cache.clear();
        self.sublist_cache.get(&provider)?.clear();
        Some(())
    }

    pub(super) fn clear_derived_sublist(&self, provider: Provider, sublist_id: &str) -> Option<()> {
        let derived = self.derived_caches(provider)?;
        derived.normal_cache.remove(sublist_id);
        derived.inverted_cache.retain(|_, value| sublist_id != value);

        Some(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CacheAction {
    ReconcileCache,
    Nothing,
}

impl CacheAction {
    /// Combine one [CacheAction] with another.
    ///
    /// So long as at least one requires some form of action that one is picked.
    pub fn combine(&self, other: CacheAction) -> CacheAction {
        if let Self::ReconcileCache = self {
            CacheAction::ReconcileCache
        } else if let Self::ReconcileCache = other {
            CacheAction::ReconcileCache
        } else {
            CacheAction::Nothing
        }
    }
}

pub struct DerivedCaches<'a> {
    pub(crate) normal_cache: &'a DashMap<SublistId, SaleInfo>,
    pub(crate) inverted_cache: &'a DashMap<ProductId, SublistId>,
}

impl<'a> DerivedCaches<'a> {
    /// Check whether the current derived caches are valid under the assumption that there are normal promotions
    ///
    /// This can become `false` if the main cache expired `promotions_cache` but the scheduled refresh job hasn't yet
    /// had a chance to run.
    pub fn are_valid(&self) -> bool {
        !self.normal_cache.is_empty() && !self.inverted_cache.is_empty()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct PromoCacheEntry<T> {
    item: T,
    inserted_at: DateTime<Utc>,
    expires: DateTime<Utc>,
}

impl<T> PromoCacheEntry<T> {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires
    }
}

impl<T> Deref for PromoCacheEntry<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<T> DerefMut for PromoCacheEntry<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}
