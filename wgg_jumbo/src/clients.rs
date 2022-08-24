use crate::ids::{Id, ProductId, PromotionId, RuntimeId, TabId};
use crate::models::{FullProductResponse, ProductList, Promotion, PromotionGroup, PromotionTabs, SortedByQuery};
use crate::Config;
use crate::Result;
use reqwest::Response;
use std::collections::HashMap;

type Query<'key, 'value> = HashMap<&'key str, &'value str>;

#[async_trait::async_trait]
pub trait BaseApi {
    fn get_config(&self) -> &Config;

    fn get_http(&self) -> &reqwest::Client;

    #[doc(hidden)]
    #[inline]
    async fn endpoint_get(&self, url_suffix: &str, payload: &Query<'_, '_>) -> Result<Response> {
        let url = self.get_config().get_full_url(url_suffix);

        let response = self.get_http().get(url).query(payload).send().await?;
        println!("URL: {}", response.url());

        Ok(response)
    }

    /// Retrieve the promotion tabs (promotion groups, aka, weekly promotions/seasonal/etc), and the associated run-times.
    async fn promotion_tabs(&self) -> Result<PromotionTabs> {
        let response = self.endpoint_get("/promotion-tabs", &Default::default()).await?;

        Ok(response.json().await?)
    }

    /// Return all the promotions, and their contained products, that are part of this promotion group.
    ///
    /// Note that the promotions returned by this endpoint are the fully detailed representations, no need for a separate
    /// query to [BaseApi::promotion].
    ///
    /// # Arguments
    /// * `store_id` - Local store Id for the specific pricing/availability.
    /// * `sorted_by` - How to sort the results, seems ineffective in the back-end at the moment.
    async fn promotion_group(
        &self,
        tab: &TabId,
        runtime: &RuntimeId,
        store_id: Option<u32>,
        sorted_by: Option<SortedByQuery>,
    ) -> Result<PromotionGroup> {
        let url = format!("/promotion-tabs/{}/{}", tab.as_ref(), runtime.as_ref());
        let store_id = store_id.map(|i| i.to_string());
        let sorted_by = sorted_by.map(|i| format!("{:?}", i));
        let query = crate::utils::build_map([("store_id", store_id.as_deref()), ("sorted_by", sorted_by.as_deref())]);
        let response = self.endpoint_get(&url, &query).await?;

        Ok(response.json().await?)
    }

    /// Retrieve the full promotion info.
    async fn promotion(&self, promotion_id: &PromotionId) -> Result<Promotion> {
        let response = self
            .endpoint_get(&format!("/promotion/{}", promotion_id), &Default::default())
            .await?;

        Ok(response.json().await?)
    }

    /// Retrieve the products for a specific promotion.
    ///
    /// Note that both `count` and `offset` have to be provided, otherwise the route would return nothing at all.
    async fn products_promotion(
        &self,
        count: u32,
        offset: u32,
        promotion_id: Option<&PromotionId>,
    ) -> Result<ProductList> {
        let count = count.to_string();
        let offset = offset.to_string();
        let promotion_id = promotion_id.map(|i| i.id());
        let query = crate::utils::build_map([
            ("count", Some(count.as_ref())),
            ("offset", Some(offset.as_ref())),
            ("promotionId", promotion_id),
        ]);
        let response = self.endpoint_get("/products", &query).await?;

        Ok(response.json().await?)
    }

    /// Retrieve all products.
    ///
    /// Note this route sends *a lot* of additional info that isn't relevant or parsed.
    /// Highly advise not using this route directly, but instead going through promotions/categories first.
    async fn products(&self, count: Option<u32>, offset: Option<u32>) -> Result<ProductList> {
        let count = count.map(|s| s.to_string());
        let offset = offset.map(|s| s.to_string());
        let query = crate::utils::build_map([("count", count.as_deref()), ("offset", offset.as_deref())]);
        let response = self.endpoint_get("/products", &query).await?;

        Ok(response.json().await?)
    }

    /// Retrieve the full details of a product.
    async fn product(&self, product_id: &ProductId) -> Result<FullProductResponse> {
        let response = self
            .endpoint_get(&format!("/products/{}", product_id), &Default::default())
            .await?;

        Ok(response.json().await?)
    }
}
