use crate::caching::get_default_provider_map;
use crate::error::Result;
use crate::models::{ProductId, Provider, SublistId, WggSaleCategory, WggSaleGroupComplete, WggSaleItem};
use crate::sale_resolver::promotions_cache::CacheAction;
use crate::{DynProvider, DynamicProviders};
use anyhow::Context;
use chrono::{DateTime, Utc};
use dashmap::try_result::TryResult;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use tracing::log;

mod promotions_cache;
pub mod scheduled;

pub use promotions_cache::PromotionsCache;

#[derive(Clone)]
pub struct SaleResolver {
    cache: PromotionsCache,
    providers: Arc<DynamicProviders>,
    pub(crate) meta_info: DashMap<Provider, ProviderMetaInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ProviderMetaInfo {
    /// This is relevant for cache behaviour.
    /// Whenever we have to perform an unscheduled full-invalidate we lose a lot of information with regard to
    /// valid sales etc.
    ///
    /// This bit can be flipped by the scheduled task to indicate that all the state is valid again.
    pub is_complete: bool,
    pub expiry: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SaleInfo {
    valid_from: DateTime<Utc>,
    valid_until: DateTime<Utc>,
    item_ids: Vec<ProductId>,
}

impl SaleResolver {
    pub(crate) fn new(providers: Arc<DynamicProviders>, previous_cache: Option<PromotionsCache>) -> Self {
        let provider_keys = providers.keys().copied();
        let cache = if let Some(mut cache) = previous_cache {
            cache.restore_from_cached_state(provider_keys.clone());
            cache.clear_expired();
            cache
        } else {
            PromotionsCache::new(provider_keys.clone())
        };

        SaleResolver {
            cache,
            meta_info: get_default_provider_map(provider_keys),
            providers,
        }
    }

    /// Retrieve all promotions for the given provider.
    ///
    /// # Caching behaviour
    ///
    /// Whenever this function has to go over the network the response will be cloned and saved until either:
    /// * The end of the current week (Sunday 23:59:59)
    /// * A given sale end-date by the [Provider]
    ///
    /// Whenever it is saved a sequence of network requests can occur (depending on the provider) for sub-lists.
    /// This is unfortunately necessary to ensure valid sale resolution.
    /// Until it is invalidated all subsequent requests will be served by this cache, save for one exception
    ///
    /// # Scheduled Refresh
    ///
    /// This sale resolver has a scheduled job which can be optionally used to check, every day at a user defined time,
    /// for cache staleness. If this is detected all stale promotions *and* their sub-lists are refreshed.
    ///
    /// Note that this can cause quite a bit of network traffic, so the `rps` config options for each [Provider] should be set sensibly!
    ///
    /// [Provider]: crate::providers::ProviderInfo
    pub async fn promotions(&self, provider: Provider) -> Result<Vec<WggSaleCategory>> {
        let result = self.cache.promotions(provider);

        match result {
            Ok(result) => Ok(result),
            Err(action) => {
                let prov = self.providers.find_provider(provider)?;
                let result = prov.promotions().await?;

                let insert_action = self.cache.insert_promotions(provider, result.clone());

                self.perform_action(action.combine(insert_action), provider).await?;

                Ok(result)
            }
        }
    }

    /// Retrieve a complete sub-list for the given provider.
    ///
    /// This function has the same caching behaviour as [promotions](Self::promotions).
    pub async fn promotion_sublist(&self, provider: Provider, sublist_id: &str) -> Result<WggSaleGroupComplete> {
        let result = self.cache.promotion_sublist(provider, sublist_id);

        match result {
            Ok(result) => Ok(result),
            Err(action) => {
                let prov = self.providers.find_provider(provider)?;
                let result = prov.promotions_sublist(sublist_id).await?;

                let insert_action = self
                    .cache
                    .insert_promotion_sublist(provider, result.clone())
                    .unwrap_or(CacheAction::Nothing);

                self.perform_action(action.combine(insert_action), provider).await?;

                Ok(result)
            }
        }
    }

    /// Retrieve a sale info (containing the [SublistId] and all associated product ids) for a given product/provider.
    ///
    /// [None] is returned if no sale was available for the given product.
    pub fn get_sale_info(&self, provider: Provider, product_id: &str) -> Option<SaleInfo> {
        let derived = self.cache.derived_caches(provider)?;
        let sale_list = derived.inverted_cache.get(product_id)?;
        Some(derived.normal_cache.get(&*sale_list)?.clone())
    }

    /// Retrieve the associated [SublistId] for the given product/provider.
    ///
    /// If there is no associated sale [None] is returned instead.
    pub fn get_sale_sublist_id(&self, provider: Provider, product_id: &str) -> Option<SublistId> {
        let derived = self.cache.derived_caches(provider)?;
        let item = derived.inverted_cache.get(product_id)?;
        Some(item.clone())
    }

    #[allow(dead_code)]
    pub fn is_part_of_sale(&self, provider: Provider, product_id: &str) -> bool {
        let Some(derived) = self.cache.derived_caches(provider) else {
            return false;
        };
        derived.inverted_cache.get(product_id).is_some()
    }

    pub(crate) async fn refresh_promotions(&self, provider: Provider) -> Result<()> {
        refresh_promotions(self, provider).await
    }

    pub(crate) fn cache(&self) -> &PromotionsCache {
        &self.cache
    }

    async fn perform_action(&self, action: CacheAction, provider: Provider) -> Result<()> {
        match action {
            CacheAction::ReconcileCache => {
                if let TryResult::Present(mut entry) = self.meta_info.try_get_mut(&provider) {
                    entry.is_complete = false;
                    drop(entry);
                    self.refresh_promotions(provider).await
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }
}

#[tracing::instrument(level = "debug", skip(sales), err)]
pub async fn refresh_promotions(sales: &SaleResolver, provider: Provider) -> Result<()> {
    // We want to force network requests so skip directly to the underlying provider.
    let external_prov = sales.providers.find_provider(provider)?;

    let (promos, (mut added, removed)) = {
        let promos = external_prov.promotions().await?;
        let current_promos = sales.cache.promotions(provider).unwrap_or_default();

        // Check if we have to do anything new.
        let diff = get_difference(current_promos.as_slice(), &promos);
        (promos, diff)
    };

    let derived_caches = sales
        .cache
        .derived_caches(provider)
        .context("Expected provider to exist which didn't")?;

    // Fallback to ensure we reconcile any brokenness
    if !derived_caches.are_valid() {
        sales.cache.clear_all_derived(provider);
        added = promos.clone();
    }

    if added.is_empty() && removed.is_empty() {
        // Completely equal, can skip doing anything for this refresh!
        log::debug!("No differences detected, returning early");
        return Ok(());
    }

    tracing::trace!(
        added_len = added.len(),
        removed_len = removed.len(),
        "Detected differences for sale refresh"
    );

    // First remove the removed/modified items
    for diff in removed {
        if let Some(id) = diff.id {
            let _ = derived_caches.normal_cache.remove(&id);
        }

        for item in diff.items {
            if let WggSaleItem::Group(limited_group) = item {
                let _ = derived_caches.normal_cache.remove(&limited_group.id);
            }
        }
    }

    // Then add the newly added sub-lists
    let results = get_sales(added, external_prov, sales).await?;
    for (id, info) in results {
        derived_caches.normal_cache.insert(id, info);
    }

    // Update the full inverted cache (TODO: Replace this with an ArcSwap)
    derived_caches.inverted_cache.clear();
    for c_item in derived_caches.normal_cache.iter() {
        for item in &c_item.item_ids {
            derived_caches.inverted_cache.insert(item.clone(), c_item.key().clone());
        }
    }

    sales.cache.insert_promotions(provider, promos);

    if let Some(mut re) = sales.meta_info.get_mut(&provider) {
        re.is_complete = true;
    }

    Ok(())
}

/// Get the differences between the two provided sets.
///
/// # Returns
/// `(added, removed)`, where modified items would be present in both.
fn get_difference(vec1: &[WggSaleCategory], vec2: &[WggSaleCategory]) -> (Vec<WggSaleCategory>, Vec<WggSaleCategory>) {
    // Can be done more efficiently with HashSets instead of an O(N*M) algorithm, but I'm guessing it doesn't matter here.
    let removed = vec1
        .iter()
        .flat_map(|x| if !vec2.contains(x) { Some(x.clone()) } else { None })
        .collect();

    let added = vec2
        .iter()
        .flat_map(|x| if !vec1.contains(x) { Some(x.clone()) } else { None })
        .collect();

    (added, removed)
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn get_sales(
    promotions: impl IntoIterator<Item = WggSaleCategory>,
    provider: &DynProvider,
    sales_resolver: &SaleResolver,
) -> Result<HashMap<SublistId, SaleInfo>> {
    let mut result: HashMap<SublistId, SaleInfo> = HashMap::new();

    for promo in promotions {
        if let Some(id) = promo.id {
            let sub_list = provider.promotions_sublist(&id).await?;
            let _ = sales_resolver
                .cache
                .insert_promotion_sublist(provider.provider(), sub_list.clone());

            let to_insert = SaleInfo {
                valid_from: sub_list.sale_info.sale_validity.valid_from,
                valid_until: sub_list.sale_info.sale_validity.valid_until,
                item_ids: sub_list.items.into_iter().map(|i| i.id).collect(),
            };

            result.insert(id, to_insert);
        }

        for item in promo.items {
            if let WggSaleItem::Group(limited_group) = item {
                let id = limited_group.id;
                let to_insert = if limited_group.items.is_empty() {
                    // In this case we're not sure if we're dealing with an *actual* empty sub-group, or whether it wasn't provided by the
                    // underlying provider. Just in case we'll request the sublist data.
                    let sub_list = provider.promotions_sublist(&id).await?;
                    let item_ids = sub_list.items.iter().map(|i| i.id.clone()).collect();
                    let to_insert = SaleInfo {
                        valid_from: sub_list.sale_info.sale_validity.valid_from,
                        valid_until: sub_list.sale_info.sale_validity.valid_until,
                        item_ids,
                    };
                    let _ = sales_resolver
                        .cache
                        .insert_promotion_sublist(provider.provider(), sub_list);

                    to_insert
                } else {
                    SaleInfo {
                        valid_from: limited_group.sale_info.sale_validity.valid_from,
                        valid_until: limited_group.sale_info.sale_validity.valid_until,
                        item_ids: limited_group.items.into_iter().map(|i| i.id).collect(),
                    }
                };

                result.insert(id, to_insert);
            }
        }
    }

    Ok(result)
}
