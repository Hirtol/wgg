pub use crate::providers::PicnicCredentials;
pub use caching::{SaleInfo, SerdeCache};
pub use error::ProviderError;
pub use providers::PICNIC_RECOMMENDED_RPS;
pub use wgg_provider::{ProvidersIter, WggProvider, WggProviderBuilder};

mod caching;
mod error;
pub mod models;
pub mod pagination;
mod providers;
mod scheduled_jobs;
mod wgg_provider;

type Result<T> = std::result::Result<T, ProviderError>;
