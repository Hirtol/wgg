use crate::models::{
    Category, Delivery, DeliverySlotQuery, DeliveryStatus, ImageSize, LoginRequest, LoginResponse, ModifyCartProduct,
    MyStore, Order, OrderStatus, PagesRoot, PartialDelivery, ProductArticle, SearchResult, Suggestion, UserInfo,
};
use anyhow::{anyhow, Context};
use md5::Digest;
use reqwest::{Response, StatusCode};
use serde::Serialize;
use std::fmt::Debug;

use reqwest::header::HeaderMap;
use std::time::Duration;

pub use crate::{config::Config, error::ApiError};

mod config;
mod error;
pub mod images;
pub mod models;

pub type Result<T> = std::result::Result<T, error::ApiError>;
pub type ProductId = str;
pub type ImageId = str;
pub type DeliverySlotId = str;
pub type DeliveryId = str;
pub type OrderId = str;
pub type ListId = str;

type Query<'a> = [(&'a str, &'a str)];

/// The root struct for accessing the `Picnic` API.
///
/// See [PicnicApi::new] or [PicnicApi::from_login] for creating a new instance.
pub struct PicnicApi {
    config: Config,
    credentials: Credentials,
    client: reqwest::Client,
}

impl PicnicApi {
    /// Create a [PicnicApi] from existing [Credentials].
    ///
    /// It is the caller's responsibility to ensure the [Credentials] are valid.
    /// Otherwise, refer to [PicnicApi::from_login].
    pub fn new(credentials: Credentials, config: Config) -> Self {
        let client = get_reqwest_client(&config.user_agent).expect("Failed to create a API Client");
        PicnicApi {
            config,
            credentials,
            client,
        }
    }

    /// Create a new [PicnicApi] instance by logging in.
    ///
    /// It is recommended to save the [Credentials] in a secure place to avoid having to log in with username/password
    /// every time. One could in the future then call [PicnicApi::new].
    pub async fn from_login(email: impl Into<String>, password: impl AsRef<str>, config: Config) -> Result<Self> {
        let credentials = Self::login_impl(&config, email.into(), password.as_ref()).await?;

        Ok(Self::new(credentials, config))
    }

    /// Return the current credentials used by the [PicnicApi].
    ///
    /// Can be useful to save separately to avoid having to log in every restart.
    pub fn credentials(&self) -> &Credentials {
        &self.credentials
    }

    /// Query all user details of the current user.
    pub async fn user_details(&self) -> Result<UserInfo> {
        let response = self.get("/user", &[]).await?;

        Ok(response.json().await?)
    }

    /// Log-in again, refreshing the internal [Credentials] in the process.
    pub async fn login(&mut self, email: impl Into<String>, password: impl AsRef<str>) -> Result<&Credentials> {
        let credentials = Self::login_impl(&self.config, email.into(), password.as_ref()).await?;
        self.credentials = credentials;

        Ok(&self.credentials)
    }

    /// Search for the provided query.
    ///
    /// Note that the last `item` in [crate::models::SearchResult] will always be a [crate::models::SearchItem::ItemSuggestionDialog].
    pub async fn search(&self, query: impl AsRef<str>) -> Result<Vec<SearchResult>> {
        if query.as_ref().is_empty() {
            return Err(ApiError::EmptySearch);
        }

        let response = self.get("/search", &[("search_term", query.as_ref())]).await?;

        Ok(response.json().await?)
    }

    /// Get a suggestion for the provided query.
    pub async fn suggestions(&self, query: impl AsRef<str>) -> Result<Vec<Suggestion>> {
        let response = self.get("/suggest", &[("search_term", query.as_ref())]).await?;

        Ok(response.json().await?)
    }

    /// Return full product info for the provided product id.
    pub async fn product(&self, product_id: impl AsRef<ProductId>) -> Result<ProductArticle> {
        let response = self.get(&format!("/articles/{}", product_id.as_ref()), &[]).await?;

        Ok(response.json().await?)
    }

    /// Retrieve the full image at the specified size.
    ///
    /// Note that no credentials are needed to retrieve these images, and can therefore be used at will.
    pub async fn image(&self, image_id: impl AsRef<ImageId>, size: ImageSize) -> Result<Vec<u8>> {
        let response = self.client.get(self.image_url(image_id, size)).send().await?;
        Ok(response.bytes().await?.into())
    }

    /// Retrieve the image url for the provided image.
    ///
    /// Note that no credentials are needed to retrieve these images, and can therefore be used at will.
    pub fn image_url(&self, image_id: impl AsRef<str>, size: ImageSize) -> String {
        format!("{}/images/{}/{}.png", self.config.static_url(), image_id.as_ref(), size)
    }

    /// Returns your store, with all categories (Promotions, Recipes, and actual categories like 'Fruit from Dutch Ground').
    ///
    /// A depth of 2 or higher ensures the first 4 items in the sub-categories are returned as well. Note that there could
    /// be more items in those categories. The presence of the `MoreButton` decorator indicates such a case.
    pub async fn categories(&self, depth: u32) -> Result<MyStore> {
        let response = self.get("/my_store", &[("depth", &depth.to_string())]).await?;

        Ok(response.json().await?)
    }

    /// Retrieves the shopping cart information of the user, including the contents.
    pub async fn shopping_cart(&self) -> Result<Order> {
        let response = self.get("/cart", &[]).await?;

        Ok(response.json().await?)
    }

    /// Adds the specified product to the cart, and returns the update state of the cart.
    pub async fn add_product_to_shopping_cart(&self, product_id: impl AsRef<ProductId>, count: u32) -> Result<Order> {
        let payload = ModifyCartProduct {
            product_id: product_id.as_ref(),
            count,
        };
        let response = self.post("/cart/add_product", &payload).await?;

        Ok(response.json().await?)
    }

    /// Removes the specified product from the cart, and returns the update state of the cart.
    pub async fn remove_product_from_shopping_cart(
        &self,
        product_id: impl AsRef<ProductId>,
        count: u32,
    ) -> Result<Order> {
        let payload = ModifyCartProduct {
            product_id: product_id.as_ref(),
            count,
        };
        let response = self.post("/cart/remove_product", &payload).await?;

        Ok(response.json().await?)
    }

    /// Clear the entire shopping cart.
    ///
    /// Returns the updated state of the cart.
    pub async fn clear_shopping_cart(&self) -> Result<Order> {
        let response = self.post("/cart/clear", &()).await?;

        Ok(response.json().await?)
    }

    /// Get all possible delivery slots
    pub async fn delivery_slots(&self) -> Result<DeliverySlotQuery> {
        let response = self.get("/cart/delivery_slots", &[]).await?;

        Ok(response.json().await?)
    }

    /// Set the selected delivery slot to the provided slot id
    ///
    /// Returns the updated cart information
    pub async fn set_delivery_slot(&self, slot_id: impl AsRef<DeliverySlotId>) -> Result<Order> {
        #[derive(Serialize)]
        struct SetSlot<'a> {
            slot_id: &'a str,
        }

        let response = self
            .post(
                "/cart/set_delivery_slot",
                &SetSlot {
                    slot_id: slot_id.as_ref(),
                },
            )
            .await?;

        Ok(response.json().await?)
    }

    /// Retrieve all deliveries ever done for the current user.
    ///
    /// Will only be partially filled in. For the complete delivery information (including order) see [PicnicApi::delivery]
    pub async fn deliveries(&self, filters: &[DeliveryStatus]) -> Result<Vec<PartialDelivery>> {
        let response = self.post("/deliveries/summary", filters).await?;

        Ok(response.json().await?)
    }

    /// Get the full details of one specific delivery, including its order.
    pub async fn delivery(&self, delivery_id: impl AsRef<DeliveryId>) -> Result<Delivery> {
        let response = self.get(&format!("/deliveries/{}", delivery_id.as_ref()), &[]).await?;

        Ok(response.json().await?)
    }

    /// Broken at the moment.
    ///
    /// Need to figure out how x-picnic-agent and x-picnic-did work.
    pub async fn delivery_position(&self, delivery_id: impl AsRef<DeliveryId>) -> Result<()> {
        // TODO: May need to include picnic headers
        let response = self
            .get(&format!("/deliveries/{}/position", delivery_id.as_ref()), &[])
            .await?;

        Ok(response.json().await?)
    }

    /// Broken at the moment.
    ///
    /// Need to figure out how x-picnic-agent and x-picnic-did work.
    pub async fn delivery_scenario(&self, delivery_id: impl AsRef<DeliveryId>) -> Result<()> {
        // TODO: May need to include picnic headers
        let response = self
            .get(&format!("/deliveries/{}/scenario", delivery_id.as_ref()), &[])
            .await?;

        Ok(response.json().await?)
    }

    /// Cancels the order with the given delivery id.
    pub async fn cancel_delivery(&self, delivery_id: impl AsRef<DeliveryId>) -> Result<()> {
        let response = self
            .post(&format!("/order/delivery/{}/cancel", delivery_id.as_ref()), &())
            .await?;

        Ok(response.json().await?)
    }

    /// Sets the rating for the provided delivery from 0 to 10.
    ///
    /// Will return 400 if a delivery already has a rating.
    pub async fn set_delivery_rating(&self, delivery_id: impl AsRef<DeliveryId>, rating: u8) -> Result<()> {
        #[derive(Serialize)]
        struct SetRating {
            rating: u8,
        }
        let response = self
            .post(
                &format!("/deliveries/{}/rating", delivery_id.as_ref()),
                &SetRating { rating },
            )
            .await?;

        Ok(response.json().await?)
    }

    /// (Re)sends the invoice email of the provided delivery
    pub async fn send_delivery_invoice_email(&self, delivery_id: impl AsRef<DeliveryId>) -> Result<()> {
        let response = self
            .post(
                &format!("/deliveries/{}/resend_invoice_email", delivery_id.as_ref()),
                &(),
            )
            .await?;

        Ok(response.json().await?)
    }

    /// Provides the status of the order (note, different from a delivery!) with the given id.
    pub async fn order_status(&self, order_id: impl AsRef<OrderId>) -> Result<OrderStatus> {
        let response = self
            .get(&format!("/cart/checkout/order/{}/status", order_id.as_ref()), &[])
            .await?;

        Ok(response.json().await?)
    }

    /// Returns all available promotions if `sublist_id` is `None`.
    ///
    /// If `sublist_id` is specified then the full list of items part of that sub-category is returned.
    ///
    /// Default `depth` is 0. When specified at `>= 2` then the first 4 items for all promotions will also be returned,
    /// even when called without a specified `sublist_id`.
    pub async fn promotions(&self) -> Result<PagesRoot> {
        let url = "/pages/promo-page-root";

        let response = self.get(url, &[]).await?;

        Ok(response.json().await?)
    }

    pub async fn promotion(&self, sublist_id: &ListId) -> Result<PagesRoot> {
        let url = "/pages/promo-group-deep-dive";
        let response = self.get(url, &[("promo_group_id", sublist_id)]).await?;

        Ok(response.json().await?)
    }

    /// Returns all lists and sub-lists.
    /// Note that this returns (almost) the exact same as the catalogue provided in [PicnicApi::categories].
    ///
    /// Default `depth` is 0.
    pub async fn pages(&self, depth: u32) -> Result<Vec<Category>> {
        let response = self.get("/pages", &[("depth", &depth.to_string())]).await?;

        Ok(response.json().await?)
    }

    /// Private login function for deduplication purposes.
    async fn login_impl(config: &Config, email: String, password: &str) -> Result<Credentials> {
        let mut hasher = md5::Md5::new();

        hasher.update(password);

        let result = hasher.finalize();
        let hex = hex::encode(result);

        let client = get_reqwest_client(&config.user_agent)?;
        let login = LoginRequest {
            key: email,
            secret: hex,
            client_id: config
                .picnic_details
                .as_ref()
                .map(|i| i.client_id.clone())
                .unwrap_or_else(|| "30100".to_string()),
            client_version: config.picnic_details.as_ref().map(|i| i.client_version.clone()),
            device_id: None,
        };

        let response = client
            .post(config.get_full_url("/user/login"))
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
            .get("x-picnic-auth")
            .ok_or_else(|| anyhow!("No picnic auth token available in response: {:#?}", response))?
            .to_str()
            .context("Failed to convert to str")?
            .to_string();
        let login_response: LoginResponse = response.json().await?;

        Ok(Credentials {
            auth_token,
            user_id: login_response.user_id,
        })
    }

    async fn get(&self, url_suffix: &str, payload: &Query<'_>) -> Result<Response> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-picnic-auth",
            (&self.credentials.auth_token)
                .try_into()
                .context("Failed to convert agent")?,
        );

        if let Some(agent) = self.config.picnic_agent() {
            headers.insert("x-picnic-agent", agent.try_into().context("Failed to convert agent")?);
        }

        let response = self
            .client
            .get(self.config.get_full_url(url_suffix))
            .headers(headers)
            .query(payload)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(response),
            StatusCode::NOT_FOUND => {
                tracing::debug!(response=?response, "Failed to resolve get request");
                Err(ApiError::NotFound(url_suffix.to_string()))
            }
            StatusCode::UNAUTHORIZED => {
                tracing::debug!(status = %response.status(), ?response, "Picnic API Error");
                Err(ApiError::AuthError)
            }
            _ => {
                tracing::warn!(status = %response.status(), ?response, "Picnic API Error");
                Err(anyhow!("Error occurred: {}", response.status()).into())
            }
        }
    }

    async fn post<T: Serialize + ?Sized>(&self, url: &str, payload: &T) -> Result<Response> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-picnic-auth",
            (&self.credentials.auth_token)
                .try_into()
                .context("Failed to convert agent")?,
        );

        if let Some(agent) = self.config.picnic_agent() {
            headers.insert("x-picnic-agent", agent.try_into().context("Failed to convert agent")?);
        }

        let response = self
            .client
            .post(self.config.get_full_url(url))
            .headers(headers)
            .json(payload)
            .send()
            .await?;

        Ok(response)
    }
}

#[derive(Clone, Debug)]
pub struct Credentials {
    pub auth_token: String,
    pub user_id: String,
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
