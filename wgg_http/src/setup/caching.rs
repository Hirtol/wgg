use crate::config::Config;
use chrono::{DateTime, Local};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use wgg_providers::SerdeCache;

pub const CACHE_NAME: &str = "cache.json";

/// Deserialize the cache and log the necessary information on failure/success.
///
/// A `None` result can imply either an outdated cache (datastructures changed), or a non-existent file (first-time).
pub async fn setup_cache(config: &Config) -> Option<SerdeCache> {
    let persist_path = full_path(&config.app.cache_dir);
    match deserialize_cache(config).await {
        Ok((cache, last_modified)) => {
            let last_modified: DateTime<Local> = last_modified.map(chrono::DateTime::from).unwrap_or_default();
            tracing::debug!(created_at=?last_modified, path=?persist_path, "Successfully restored the data cache.");
            Some(cache)
        }
        Err(_) => {
            tracing::debug!(path=?persist_path, "Failed to load data cache, either it didn't exist or was outdated.");
            None
        }
    }
}

/// Serializes the given `cache` to disk and log the necessary information on failure/success.
pub async fn teardown_cache(cache: SerdeCache, config: &Config) {
    let persist_path = full_path(&config.app.cache_dir);
    if serialize_cache(cache, config).await.is_ok() {
        tracing::debug!(path=?persist_path, "Successfully persisted the data cache.");
    } else {
        tracing::debug!(path=?persist_path, "Failed to persist data cache.");
    }
}

/// Serializes the provided [SerdeWggCache] to a JSON file dropped in the [Config]'s `cache_dir`
async fn serialize_cache(cache: SerdeCache, config: &Config) -> anyhow::Result<()> {
    let json = serde_json::to_string(&cache)?;

    std::fs::create_dir_all(&config.app.cache_dir)?;

    Ok(std::fs::write(full_path(&config.app.cache_dir), json)?)
}

/// Deserializes the [wgg_providers::WggProvider] cache and returns it, alongside the last modified time, if available.
async fn deserialize_cache(config: &Config) -> anyhow::Result<(SerdeCache, Option<SystemTime>)> {
    let cache_file = full_path(&config.app.cache_dir);
    let last_modified = tokio::fs::metadata(&cache_file).await?.modified().ok();
    let contents = tokio::fs::read(cache_file).await?;

    Ok((serde_json::from_slice(&contents)?, last_modified))
}

fn full_path(dir_path: &Path) -> PathBuf {
    dir_path.join(CACHE_NAME)
}
