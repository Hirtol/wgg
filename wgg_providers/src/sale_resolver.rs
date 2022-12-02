use crate::caching::{get_default_provider_map, PromotionsCache};
use crate::models::{ProductId, Provider, SaleValidity, SublistId, WggDecorator, WggSaleCategory, WggSaleItem};
use crate::providers::ProviderInfo;
use crate::WggProvider;
use anyhow::Context;
use chrono::{DateTime, Datelike, Utc, Weekday};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct SaleResolver {
    pub(crate) cache: PromotionsCache,
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
    pub fn new(providers: impl Iterator<Item = Provider> + Clone, previous_cache: Option<PromotionsCache>) -> Self {
        let cache = if let Some(mut cache) = previous_cache {
            cache.restore_from_cached_state(providers.clone());
            cache.clear_expired();
            cache
        } else {
            PromotionsCache::new(providers.clone())
        };

        SaleResolver {
            cache,
            meta_info: get_default_provider_map(providers),
        }
    }

    pub fn get_sale_info(&self, provider: Provider, product_id: &str) -> Option<SaleInfo> {
        let derived = self.cache.derived_caches(provider)?;
        let sale_list = derived.inverted_cache.get(product_id)?;
        Some(derived.normal_cache.get(&*sale_list)?.clone())
    }

    pub async fn refresh_promotions(&self, provider: Provider, wgg_providers: &WggProvider) -> crate::Result<()> {
        refresh_promotions(wgg_providers, provider).await
    }
}

pub mod scheduled {
    use crate::sale_resolver::refresh_promotions;
    use crate::WggProvider;
    use std::sync::Arc;
    use wgg_scheduler::schedule::Schedule;
    use wgg_scheduler::Job;

    pub fn get_promotions_refresh_job(schedule: Schedule, providers: Arc<WggProvider>) -> Job {
        Job::new(schedule, move |_, _| {
            let provs = providers.clone();
            Box::pin(async move {
                let span = tracing::span!(tracing::Level::DEBUG, "Scheduled Job - Promotion Data");
                let _enter = span.enter();

                for provider in provs.active_providers() {
                    let real_provider = provider.provider();
                    tracing::debug!(provider=?real_provider, "Refreshing promotion data for provider");
                    refresh_promotions(&provs, real_provider).await?
                }
                Ok(())
            })
        })
        .expect("Invalid schedule provided")
    }
}

pub async fn refresh_promotions(providers: &WggProvider, provider: Provider) -> crate::Result<()> {
    let WggProvider { sales, .. } = providers;
    // We want to force network requests so skip directly to the underlying provider.
    let external_prov = providers.find_provider(provider)?;

    let (promos, (added, removed)) = {
        let promos = external_prov.promotions().await?;
        let current_promos = sales.cache.promotions(provider).unwrap_or_default();

        // Check if we have to do anything new.
        let diff = get_difference(&promos, current_promos.as_slice());
        (promos, diff)
    };

    if added.is_empty() && removed.is_empty() {
        // Completely equal, can skip doing anything for this refresh!
        return Ok(());
    }

    // First remove the removed or modified items
    let derived_caches = sales
        .cache
        .derived_caches(provider)
        .context("Expected provider to exist which didn't")?;
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
    let results = get_sales(added, external_prov).await?;
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

pub async fn get_sales(
    promotions: impl IntoIterator<Item = WggSaleCategory>,
    provider: &(dyn ProviderInfo + Send + Sync),
) -> crate::Result<HashMap<SublistId, SaleInfo>> {
    let mut result: HashMap<SublistId, SaleInfo> = HashMap::new();

    for promo in promotions {
        if let Some(id) = promo.id {
            let sub_list = provider.promotions_sublist(&id).await?;
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
