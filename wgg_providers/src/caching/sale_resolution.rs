use crate::models::{ProductId, Provider, SaleValidity, SublistId, WggDecorator, WggSaleCategory, WggSaleItem};
use crate::providers::ProviderInfo;
use crate::Result;
use crate::WggProvider;
use anyhow::Context;
use chrono::{DateTime, Datelike, Utc, Weekday};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type ProviderMap<T> = HashMap<Provider, T>;

#[derive(Serialize, Deserialize, Clone)]
pub struct SaleResolver {
    pub(crate) promotions_cache: DashMap<Provider, Vec<WggSaleCategory>>,
    // pub(crate) sublist_cache: ProviderMap<DashMap<SublistId, WggSaleGroupComplete>>,
    pub(crate) normal_cache: ProviderMap<DashMap<SublistId, SaleInfo>>,
    pub(crate) inverted_cache: ProviderMap<DashMap<ProductId, SublistId>>,
    pub(crate) meta_info: DashMap<Provider, ProviderMetaInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ProviderMetaInfo {
    pub is_complete: bool,
    pub is_fetching: bool,
    pub expiry: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SaleInfo {
    valid_from: DateTime<Utc>,
    valid_until: DateTime<Utc>,
    item_ids: Vec<ProductId>,
}

impl SaleResolver {
    pub fn new(providers: impl Iterator<Item = Provider> + Clone) -> Self {
        fn get_default_provider_map<T: Default, B: FromIterator<(Provider, T)>>(
            providers: impl Iterator<Item = Provider>,
        ) -> B {
            providers.map(|i| (i, T::default())).collect()
        }

        SaleResolver {
            promotions_cache: get_default_provider_map(providers.clone()),
            // sublist_cache: get_default_provider_map(providers.clone()),
            normal_cache: get_default_provider_map(providers.clone()),
            inverted_cache: get_default_provider_map(providers.clone()),
            meta_info: get_default_provider_map(providers),
        }
    }

    pub fn get_sale_info(&self, provider: Provider, product_id: &str) -> Option<SaleInfo> {
        let sale_list = self.inverted_cache.get(&provider)?.get(product_id)?;
        Some(self.normal_cache.get(&provider)?.get(&*sale_list)?.clone())
    }

    pub fn get_promotions(&self, provider: Provider) -> Option<Vec<WggSaleCategory>> {
        let now = Utc::now();
        let metadata = self.meta_info.get(&provider)?;

        if now >= metadata.expiry {
            None
        } else {
            Some(self.promotions_cache.get(&provider)?.clone())
        }
    }

    pub fn insert_promotions(&self, provider: Provider, promos: Vec<WggSaleCategory>) {
        // Would have to refresh the other caches as well?
        todo!()
    }

    pub async fn refresh_promotions(&self, provider: Provider, wgg_providers: &WggProvider) -> Result<()> {
        refresh_promotions(wgg_providers, provider).await
    }
}

pub mod scheduled {
    use super::refresh_promotions;
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

pub async fn refresh_promotions(providers: &WggProvider, provider: Provider) -> Result<()> {
    let WggProvider { sales, .. } = providers;
    // We want to force network requests so skip directly to the underlying provider.
    let external_prov = providers.find_provider(provider)?;

    let (promos, (added, removed)) = {
        let promos = external_prov.promotions().await?;
        let current_promos = sales
            .promotions_cache
            .get(&provider)
            .context("Expected provider didn't exist")?;

        // Check if we have to do anything new.
        let diff = get_difference(&promos, current_promos.as_slice());
        (promos, diff)
    };

    if added.is_empty() && removed.is_empty() {
        // Completely equal, can skip doing anything for this refresh!
        return Ok(());
    }

    // First remove the removed or modified items
    let normal_cache = sales
        .normal_cache
        .get(&provider)
        .context("Expected provider didn't exist")?;
    for diff in removed {
        if let Some(id) = diff.id {
            let _ = normal_cache.remove(&id);
        }

        for item in diff.items {
            if let WggSaleItem::Group(limited_group) = item {
                let _ = normal_cache.remove(&limited_group.id);
            }
        }
    }

    // Then add the newly added sub-lists
    let results = get_sales(added, external_prov).await?;
    for (id, info) in results {
        normal_cache.insert(id, info);
    }

    // Update the full inverted cache (TODO: Replace this with an ArcSwap)
    let inverted_cache = sales
        .inverted_cache
        .get(&provider)
        .context("Expected provider didn't exist")?;

    inverted_cache.clear();
    for c_item in normal_cache.iter() {
        for item in &c_item.item_ids {
            inverted_cache.insert(item.clone(), c_item.key().clone());
        }
    }

    sales.promotions_cache.insert(provider, promos);

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
) -> Result<HashMap<SublistId, SaleInfo>> {
    let mut result: HashMap<SublistId, SaleInfo> = HashMap::new();

    for promo in promotions {
        if let Some(id) = promo.id {
            let sub_list = provider.promotions_sublist(&id).await?;
            let sale_validity = get_sale_validity(sub_list.decorators);

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
                let sale_validity = get_sale_validity(limited_group.decorators);

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

fn get_sale_validity(decorators: impl IntoIterator<Item = WggDecorator>) -> SaleValidity {
    decorators
        .into_iter()
        .flat_map(|i| match i {
            WggDecorator::SaleValidity(valid) => Some(valid),
            _ => None,
        })
        .next()
        .unwrap_or_else(get_guessed_sale_validity)
}

fn get_guessed_sale_validity() -> SaleValidity {
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
