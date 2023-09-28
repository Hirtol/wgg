use reqwest::{Client, Response};
use serde::Serialize;
use std::collections::HashMap;

use crate::models::{LoginRequest, UserResponse};
use anyhow::anyhow;
use std::time::Duration;

pub use crate::{base_client::BaseApi, config::Config, error::ApiError};

mod base_client;
mod config;
mod error;
pub mod ids;
pub mod models;
mod utils;

pub type Result<T> = std::result::Result<T, ApiError>;
type Query<'key, 'value> = HashMap<&'key str, &'value str>;

/// The interface to the `Jumbo` API without authenticated routes.
pub struct BaseJumboApi {
    config: Config,
    client: reqwest::Client,
}

/// The root struct for accessing the full `Jumbo` API.
/// Contains both the authenticated and unauthenticated routes.
///
/// See [FullJumboApi::new] or [FullJumboApi::from_login] for creating a new instance.
pub struct FullJumboApi {
    config: Config,
    credentials: Credentials,
    client: reqwest::Client,
}

impl BaseJumboApi {
    /// Create a new unauthenticated interface to the Jumbo API.
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
            .ok_or_else(|| ApiError::LoginFailed(format!("No Jumbo auth token available in response: {response:#?}")))?
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

    /// Return the current credentials used by the [FullJumboApi].
    ///
    /// Can be useful to save separately to avoid having to log in every restart.
    pub fn credentials(&self) -> &Credentials {
        &self.credentials
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

    #[allow(dead_code)]
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
