use crate::config::Config;
use reqwest::{Client, Response, Url};
use serde::Serialize;

use crate::clients::BaseApi;
use std::time::Duration;

pub mod clients;
pub mod config;
pub mod error;
pub mod ids;
pub mod models;
mod utils;

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

    async fn get(&self, url_suffix: &str, payload: &Query<'_>) -> Result<Response> {
        let response = self
            .client
            .get(self.config.get_full_url(url_suffix))
            .header("x-jumbo-token", &self.credentials.auth_token)
            .query(payload)
            .send()
            .await?;

        Ok(response)
    }

    async fn post<T: Serialize + ?Sized>(&self, url: &str, payload: &T) -> Result<Response> {
        let response = self
            .client
            .post(self.config.get_full_url(url))
            .header("x-jumbo-token", &self.credentials.auth_token)
            .json(payload)
            .send()
            .await?;

        Ok(response)
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
    use crate::ids::{ProductId, PromotionId};
    use crate::models::SortedByQuery;
    use crate::{BaseApi, BaseJumboApi};

    #[tokio::test]
    pub async fn testo() {
        let api = BaseJumboApi::new(Default::default());

        // let response = api.promotion_tabs().await.unwrap();
        // let promotion_id: PromotionId = "1222049-A-1".parse().unwrap();
        // let response = api.products_promotion(10, 0, Some(&promotion_id)).await.unwrap();
        let product_id: ProductId = "441710STK".parse().unwrap();
        // let response = api.search("komkommer", None, None).await.unwrap();
        let response = api.product(&product_id).await.unwrap();
        println!("{:#?}", response)
    }
}
