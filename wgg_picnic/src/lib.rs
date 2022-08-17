use crate::config::Config;
use crate::error::ApiError;
use crate::models::{
    ImageSize, LoginRequest, LoginResponse, MyStore, ProductResult, SearchResult, Suggestion, UserInfo,
};
use anyhow::anyhow;
use md5::Digest;
use reqwest::{Response, Url};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::time::Duration;

pub mod config;
pub mod error;
pub mod models;

pub type Result<T> = std::result::Result<T, error::ApiError>;
type Query<'a> = [(&'a str, &'a str)];

pub struct PicnicApi {
    config: Config,
    credentials: Credentials,
    client: reqwest::Client,
}

impl PicnicApi {
    pub fn new(credentials: Credentials, config: Config) -> Self {
        let client = get_reqwest_client(&config.user_argent).expect("Failed to create a API Client");
        PicnicApi {
            config,
            credentials,
            client,
        }
    }

    pub async fn from_login<T: AsRef<str>>(username: impl Into<String>, password: T, config: Config) -> Result<Self> {
        let mut hasher = md5::Md5::new();

        hasher.update(password.as_ref());

        let result = hasher.finalize();
        let hex = hex::encode(result);

        let client = get_reqwest_client(&config.user_argent)?;
        let login = LoginRequest {
            key: username.into(),
            secret: hex,
            client_id: 1,
        };

        let response = client.post(config.url.clone()).json(&login).send().await?;

        if response.status().is_client_error() {
            return Err(ApiError::LoginFailed(format!(
                "Status: {} - Body: {}",
                response.status(),
                response.text().await?
            )));
        }

        let auth_token = response
            .headers()
            .get("x-picnic-auth")
            .ok_or_else(|| anyhow!("No picnic auth token available in response: {:#?}", response))?
            .to_str()
            .map_err(|e| anyhow!(e))?
            .to_string();
        let login_response: LoginResponse = response.json().await?;

        let credentials = Credentials {
            auth_token,
            user_id: login_response.user_id,
        };

        Ok(Self::new(credentials, config))
    }

    /// Query all user details of the current user.
    pub async fn user_details(&self) -> Result<UserInfo> {
        let response = self.get("/user", &[]).await?;

        Ok(response.json().await?)
    }

    /// Search for the provided query.
    ///
    /// Note that the last `item` in [crate::models::SearchResult] will always be a [crate::models::SearchItem::ItemSuggestionDialog].
    pub async fn search(&self, query: impl AsRef<str>) -> Result<Vec<SearchResult>> {
        let encoded_term = urlencoding::encode(query.as_ref());
        let response = self.get("/search", &[("search_term", encoded_term.as_ref())]).await?;

        Ok(response.json().await?)
    }

    /// Get a suggestion for the provided query.
    pub async fn suggestions(&self, query: impl AsRef<str>) -> Result<Vec<Suggestion>> {
        let encoded_term = urlencoding::encode(query.as_ref());
        let response = self.get("/suggest", &[("search_term", encoded_term.as_ref())]).await?;

        Ok(response.json().await?)
    }

    /// Return full product info for the provided product id.
    pub async fn product(&self, product_id: impl AsRef<str>) -> Result<ProductResult> {
        let response = self.get(&format!("/product/{}", product_id.as_ref()), &[]).await?;

        Ok(response.json().await?)
    }

    /// Retrieve the full image at the specified size.
    ///
    /// Note that no credentials are needed to retrieve these images, and can therefore be used at will.
    pub async fn image(&self, image_id: impl AsRef<str>, size: ImageSize) -> Result<Vec<u8>> {
        let response = self.client.get(self.image_url(image_id, size)).send().await?;
        Ok(response.bytes().await?.into())
    }

    /// Retrieve the image url for the provided image.
    ///
    /// Note that no credentials are needed to retrieve these images, and can therefore be used at will.
    pub fn image_url(&self, image_id: impl AsRef<str>, size: ImageSize) -> Url {
        let url = format!("{}/images/{}/{}.png", self.config.static_url(), image_id.as_ref(), size);
        // We know that the URL will be valid
        url.parse().unwrap()
    }

    pub async fn categories(&self, depth: u32) -> Result<MyStore> {
        let response = self
            .get(&format!("/my_store"), &[("depth", &depth.to_string())])
            .await?;

        Ok(response.json().await?)
    }

    async fn get(&self, url_suffix: &str, payload: &Query<'_>) -> Result<Response> {
        let response = self
            .client
            .get(self.get_full_url(url_suffix))
            .header("x-picnic-auth", &self.credentials.auth_token)
            .query(payload)
            .send()
            .await?;

        Ok(response)
    }

    async fn post<T: Serialize>(&self, url: &str, payload: &T) -> Result<Response> {
        let response = self
            .client
            .post(self.get_full_url(url))
            .header("x-picnic-auth", &self.credentials.auth_token)
            .json(payload)
            .send()
            .await?;

        Ok(response)
    }

    fn get_full_url(&self, suffix: &str) -> String {
        format!("{}{}", self.config.url(), suffix)
    }
}
#[derive(Clone)]
pub struct Credentials {
    auth_token: String,
    user_id: String,
}

impl Credentials {
    pub fn new(auth_token: String, user_id: String) -> Self {
        Self { auth_token, user_id }
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
    use crate::{Config, Credentials, PicnicApi};

    #[tokio::test]
    pub async fn test_setup() {
        let auth_cred = dotenv::var("PICNIC_AUTH_TOKEN").unwrap();
        let user_id = dotenv::var("PICNIC_USER_ID").unwrap();

        let cred = Credentials {
            auth_token: auth_cred,
            user_id,
        };

        let api = PicnicApi::new(cred, Config::default());
        let response = api.categories(0).await.unwrap();
        println!("Response: {:#?}", response);
    }
}
