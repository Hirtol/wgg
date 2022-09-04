use crate::models::{Autocomplete, Provider, SearchItem};
use wgg_jumbo::BaseJumboApi;
use wgg_picnic::PicnicApi;

pub use error::ProviderError;

mod error;
pub mod models;

mod jumbo_bridge;
mod picnic_bridge;

type Result<T> = std::result::Result<T, ProviderError>;

#[async_trait::async_trait]
pub trait ProviderInfo {
    async fn autocomplete(&self, query: &str) -> Result<Vec<Autocomplete>>;

    async fn search(&self, query: &str) -> Result<Vec<SearchItem>>;
}

pub struct WggProvider {
    picnic: PicnicApi,
    jumbo: BaseJumboApi,
}

impl WggProvider {
    /// Create a new provider instance with a username and password combo.
    ///
    /// Ideally one would persist the credentials in a safe location to avoid having to log-in to Picnic every restart.
    pub async fn new(picnic_username: &str, picnic_password: &str) -> Result<Self> {
        let picnic = PicnicApi::from_login(picnic_username, picnic_password, Default::default()).await?;

        Ok(Self {
            picnic,
            jumbo: BaseJumboApi::new(Default::default()),
        })
    }

    /// Create a new provider from pre-existing *valid* [wgg_picnic::Credentials].
    pub fn from_credentials(picnic_credentials: wgg_picnic::Credentials) -> Result<Self> {
        Ok(Self {
            picnic: PicnicApi::new(picnic_credentials, Default::default()),
            jumbo: BaseJumboApi::new(Default::default()),
        })
    }
}
