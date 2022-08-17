use crate::config::Config;
use crate::error::ApiError;
use crate::models::{
    Category, DeliveryStatus, ImageSize, LoginRequest, LoginResponse, ModifyCartProduct, MyStore, Order, ProductResult,
    SearchResult, SubCategory, Suggestion, UserInfo,
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
pub type ProductId = str;
pub type ImageId = str;
pub type DeliverySlotId = str;
pub type DeliveryId = str;
pub type OrderId = str;
pub type ListId = str;

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
    pub async fn product(&self, product_id: impl AsRef<ProductId>) -> Result<ProductResult> {
        let response = self.get(&format!("/product/{}", product_id.as_ref()), &[]).await?;

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
    pub fn image_url(&self, image_id: impl AsRef<str>, size: ImageSize) -> Url {
        let url = format!("{}/images/{}/{}.png", self.config.static_url(), image_id.as_ref(), size);
        // We know that the URL will be valid
        url.parse().unwrap()
    }

    /// Returns your store, with all categories (Promotions, Recipes, and actual categories like 'Fruit from Dutch Ground').
    ///
    /// A depth of 2 or higher ensures the first 4 items in the sub-categories are returned as well. Note that there could
    /// be more items in those categories. The presence of the `MoreButton` decorator indicates such a case.
    pub async fn categories(&self, depth: u32) -> Result<MyStore> {
        let response = self
            .get(&format!("/my_store"), &[("depth", &depth.to_string())])
            .await?;

        Ok(response.json().await?)
    }

    pub async fn shopping_cart(&self) -> Result<Order> {
        let response = self.get("/cart", &[]).await?;

        Ok(response.json().await?)
    }

    pub async fn add_product_to_shopping_cart(&self, product_id: impl AsRef<ProductId>, count: u32) -> Result<Order> {
        let payload = ModifyCartProduct {
            product_id: product_id.as_ref(),
            count,
        };
        let response = self.post("/cart/add_product", &payload).await?;

        Ok(response.json().await?)
    }

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

    pub async fn clear_shopping_cart(&self) -> Result<Order> {
        let response = self.post("/cart/clear", &()).await?;

        Ok(response.json().await?)
    }

    pub async fn delivery_slots(&self) -> Result<Vec<()>> {
        let response = self.get("/cart/delivery_slots", &[]).await?;

        Ok(response.json().await?)
    }

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

    pub async fn deliveries(&self, filters: &[DeliveryStatus]) -> Result<Vec<()>> {
        let response = self.post("/deliveries/summary", filters).await?;

        Ok(response.json().await?)
    }

    pub async fn delivery(&self, delivery_id: impl AsRef<DeliveryId>) -> Result<()> {
        let response = self.get(&format!("/deliveries/{}", delivery_id.as_ref()), &[]).await?;

        Ok(response.json().await?)
    }

    pub async fn delivery_position(&self, delivery_id: impl AsRef<DeliveryId>) -> Result<()> {
        // TODO: May need to include picnic headers
        let response = self
            .get(&format!("/deliveries/{}/position", delivery_id.as_ref()), &[])
            .await?;

        Ok(response.json().await?)
    }

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
    pub async fn order_status(&self, order_id: impl AsRef<OrderId>) -> Result<()> {
        let response = self
            .get(&format!("/cart/checkout/order/{}/status", order_id.as_ref()), &[])
            .await?;

        Ok(response.json().await?)
    }

    /// Returns all lists and sub-lists.
    /// Note that this returns (almost) the exact same as the catalogue provided in [PicnicApi::categories].
    ///
    /// Default `depth` is 0.
    pub async fn lists(&self, depth: u32) -> Result<Vec<Category>> {
        let response = self.get("/lists", &[("depth", &depth.to_string())]).await?;

        Ok(response.json().await?)
    }

    /// Retrieves the sub-lists of a list if no `sublist_id` was provided.
    /// Optionally, if given a `depth >= 2` will also include the articles in those sub-lists.
    ///
    /// Retrieves the articles of a sublist if the `sublist_id` was given.
    pub async fn list(
        &self,
        list_id: impl AsRef<ListId>,
        sublist_id: Option<impl AsRef<ListId>>,
        depth: u32,
    ) -> Result<Vec<SubCategory>> {
        let url = format!("/lists/{}", list_id.as_ref());
        let response = if let Some(sublist) = sublist_id {
            self.get(
                &url,
                &[("sublist", sublist.as_ref()), ("depth", depth.to_string().as_ref())],
            )
            .await
        } else {
            self.get(&url, &[("depth", depth.to_string().as_ref())]).await
        }?;

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

    async fn post<T: Serialize + ?Sized>(&self, url: &str, payload: &T) -> Result<Response> {
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
    use crate::{Config, Credentials, ModifyCartProduct, PicnicApi};

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
