use crate::config::Config;
use reqwest::{Client, Response, Url};
use serde::Serialize;

use crate::clients::BaseApi;
use std::time::Duration;

pub mod clients;
pub mod config;
pub mod error;
pub mod models;

pub type Result<T> = std::result::Result<T, error::ApiError>;
pub type ProductId = str;
pub type ImageId = str;
pub type DeliverySlotId = str;
pub type DeliveryId = str;
pub type OrderId = str;
pub type ListId = str;

type Query<'a> = [(&'a str, &'a str)];

pub struct BaseJumboApi {
    config: Config,
    client: reqwest::Client,
}

impl BaseJumboApi {
    pub fn new(config: Config) -> Self {
        Self {
            client: get_reqwest_client(&config.user_agent).unwrap(),
            config,
        }
    }
}

impl BaseApi for BaseJumboApi {
    fn get_config(&self) -> &Config {
        &self.config
    }

    fn get_http(&self) -> &Client {
        &self.client
    }
}

/// The root struct for accessing the `Picnic` API.
///
/// See [PicnicApi::new] or [PicnicApi::from_login] for creating a new instance.
pub struct FullJumboApi {
    config: Config,
    credentials: Credentials,
    client: reqwest::Client,
}

impl FullJumboApi {
    /// Create a [FullJumboApi] from existing [Credentials].
    ///
    /// It is the caller's responsibility to ensure the [Credentials] are valid.
    /// Otherwise, refer to [FullJumboApi::from_login].
    pub fn new(credentials: Credentials, config: Config) -> Self {
        let client = get_reqwest_client(&config.user_agent).expect("Failed to create a API Client");
        FullJumboApi {
            config,
            credentials,
            client,
        }
    }

    fn get_full_url(&self, suffix: &str) -> String {
        self.config.get_full_url(suffix)
    }
}

impl BaseApi for FullJumboApi {
    fn get_config(&self) -> &Config {
        &self.config
    }

    fn get_http(&self) -> &Client {
        &self.client
    }
}

#[derive(Clone)]
pub struct Credentials {
    pub auth_token: String,
}

impl Credentials {
    pub fn new(auth_token: String) -> Self {
        Self { auth_token }
    }
}

fn get_reqwest_client(user_agent: &str) -> anyhow::Result<reqwest::Client> {
    Ok(reqwest::ClientBuilder::default()
        .timeout(Duration::from_secs(10))
        .tcp_keepalive(Duration::from_secs(20))
        .gzip(true)
        .user_agent(user_agent)
        .build()?)
}

mod tests {
    use crate::{BaseApi, BaseJumboApi};

    #[tokio::test]
    pub async fn testo() {
        let api = BaseJumboApi::new(Default::default());

        let response = api.promotion_tabs().await.unwrap();

        println!("{:#?}", response)
    }
}
