use crate::caching::get_default_provider_map;
use crate::models::{
    ProductId, Provider, SaleValidity, SublistId, WggDecorator, WggSaleCategory, WggSaleGroupComplete, WggSaleItem,
};
use crate::sale_resolver::promotions_cache::InsertionAction;
use crate::ProviderError;
use crate::{DynProvider, DynamicProviders};
use anyhow::Context;
use chrono::{DateTime, Datelike, Utc, Weekday};
use dashmap::try_result::TryResult;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
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

    pub async fn promotions(&self, provider: Provider) -> Option<Vec<WggSaleCategory>> {
        let result = self.cache.promotions(provider);

        match result {
            Ok(result) => Some(result),
            Err(action) => {
                let _ = self.perform_action(action, provider).await;
                None
            }
        }
    }

    pub async fn promotion_sublist(&self, provider: Provider, sublist_id: &str) -> Option<WggSaleGroupComplete> {
        let action = self.cache.promotion_sublist(provider, sublist_id);

        match action {
            Ok(result) => Some(result),
            Err(action) => {
                let _ = self.perform_action(action, provider).await;
                None
            }
        }
    }

    pub fn get_sale_info(&self, provider: Provider, product_id: &str) -> Option<SaleInfo> {
        let derived = self.cache.derived_caches(provider)?;
        let sale_list = derived.inverted_cache.get(product_id)?;
        Some(derived.normal_cache.get(&*sale_list)?.clone())
    }

    pub(crate) async fn insert_promotions(&self, provider: Provider, promos: Vec<WggSaleCategory>) {
        let action = self.cache.insert_promotions(provider, promos);
        let _ = self.perform_action(action, provider).await;
    }

    pub(crate) async fn insert_promotion_sublist(&self, provider: Provider, promo: WggSaleGroupComplete) -> Option<()> {
        let action = self.cache.insert_promotion_sublist(provider, promo)?;
        let _ = self.perform_action(action, provider).await;
        Some(())
    }

    pub(crate) async fn refresh_promotions(&self, provider: Provider) -> crate::error::Result<()> {
        refresh_promotions(self, provider).await
    }

    pub(crate) fn cache(&self) -> &PromotionsCache {
        &self.cache
    }

    async fn perform_action(&self, action: InsertionAction, provider: Provider) -> crate::error::Result<()> {
        match action {
            InsertionAction::ReconcileCache => {
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
pub async fn refresh_promotions(sales: &SaleResolver, provider: Provider) -> crate::error::Result<()> {
    // We want to force network requests so skip directly to the underlying provider.
    let external_prov = sales.providers.find_provider(provider)?;

    let (promos, (mut added, removed)) = {
        let promos = external_prov.promotions().await?;
        let current_promos = sales.cache.promotions(provider).unwrap_or_default();

        // Check if we have to do anything new.
        let diff = get_difference(&promos, current_promos.as_slice());
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
        ad_len = added.len(),
        rem_len = removed.len(),
        "Detected differences for sale refresh"
    );

    // First remove the removed or modified items
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
    let results = get_sales(added, external_prov.deref(), sales).await?;
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
) -> crate::error::Result<HashMap<SublistId, SaleInfo>> {
    let mut result: HashMap<SublistId, SaleInfo> = HashMap::new();

    for promo in promotions {
        if let Some(id) = promo.id {
            let sub_list = provider.promotions_sublist(&id).await?;
            sales_resolver
                .cache
                .insert_promotion_sublist(provider.provider(), sub_list.clone());

            let sale_validity = get_sale_validity(&sub_list.decorators);

            let to_insert = SaleInfo {
                valid_from: sale_validity.valid_from,
                valid_until: sale_validity.valid_until,
                item_ids: sub_list.items.into_iter().map(|i| i.id).collect(),
            };

            result.insert(id, to_insert);
        }

        for item in promo.items {
            if let WggSaleItem::Group(limited_group) = item {
                let id = limited_group.id;
                let sale_validity = get_sale_validity(&limited_group.decorators);

                let to_insert = SaleInfo {
                    valid_from: sale_validity.valid_from,
                    valid_until: sale_validity.valid_until,
                    item_ids: limited_group.items.into_iter().map(|i| i.id).collect(),
                };

                result.insert(id, to_insert);
            }
        }
    }

    Ok(result)
}

pub fn get_sale_validity<'a>(decorators: impl IntoIterator<Item = &'a WggDecorator>) -> Cow<'a, SaleValidity> {
    decorators
        .into_iter()
        .flat_map(|i| match i {
            WggDecorator::SaleValidity(valid) => Some(Cow::Borrowed(valid)),
            _ => None,
        })
        .next()
        .unwrap_or_else(|| Cow::Owned(get_guessed_sale_validity()))
}

pub fn get_guessed_sale_validity() -> SaleValidity {
    let now = Utc::now();

    // We assume a sale is valid until the very end of this week
    let sunday =
        chrono::NaiveDate::from_isoywd(now.iso_week().year(), now.iso_week().week(), Weekday::Sun).and_hms(23, 59, 59);
    let valid_until: DateTime<Utc> = DateTime::from_local(sunday, Utc);

    SaleValidity {
        valid_from: now,
        valid_until,
    }
}
