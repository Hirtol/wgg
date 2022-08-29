use crate::config::Config;
use reqwest::{Client, Response};
use serde::Serialize;
use std::collections::HashMap;

use crate::clients::BaseApi;
use crate::error::ApiError;
use crate::models::{LoginRequest, UserResponse};
use anyhow::anyhow;
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

type Query<'key, 'value> = HashMap<&'key str, &'value str>;

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

    /// Create a new [JumboApi] instance by logging in.
    ///
    /// It is recommended to save the [Credentials] in a secure place to avoid having to log in with username/password
    /// every time. One could in the future then call [JumboApi::new].
    pub async fn from_login(username: impl Into<String>, password: impl Into<String>, config: Config) -> Result<Self> {
        let client = get_reqwest_client(&config.user_agent)?;
        let login = LoginRequest {
            username: username.into(),
            password: password.into(),
        };

        let response = client
            .post(config.get_full_url("/users/login"))
            .json(&login)
            .send()
            .await?;

        if response.status().is_client_error() {
            return Err(ApiError::LoginFailed(format!(
                "Status: {} - Body: {}",
                response.status(),
                response.text().await?
            )));
        }

        let auth_token = response
            .headers()
            .get("x-jumbo-token")
            .ok_or_else(|| {
                ApiError::LoginFailed(format!("No Jumbo auth token available in response: {:#?}", response))
            })?
            .to_str()
            .map_err(|e| anyhow!(e))?
            .to_string();

        let credentials = Credentials { auth_token };

        Ok(Self::new(credentials, config))
    }

    /// Return all user details associated with this account.
    pub async fn me(&self) -> Result<UserResponse> {
        let response = self.get("/users/me", &Default::default()).await?;

        Ok(response.json().await?)
    }

    async fn get(&self, url_suffix: &str, payload: &Query<'_, '_>) -> Result<Response> {
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

#[cfg(test)]
mod tests {
    use crate::clients::BaseApi;
    use crate::ids::ProductId;
    use crate::BaseJumboApi;

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
