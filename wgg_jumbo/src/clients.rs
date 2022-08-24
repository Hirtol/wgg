use crate::ids::{RuntimeId, TabId};
use crate::models::{PromotionGroup, PromotionTabs, SortedByQuery};
use crate::Config;
use crate::Result;
use reqwest::Response;

type Query<'a> = [(&'a str, Option<&'a str>)];

#[async_trait::async_trait]
pub trait BaseApi {
    fn get_config(&self) -> &Config;

    fn get_http(&self) -> &reqwest::Client;

    #[doc(hidden)]
    #[inline]
    async fn endpoint_get(&self, url_suffix: &str, payload: &Query<'_>) -> Result<Response> {
        let url = self.get_config().get_full_url(url_suffix);

        let response = self.get_http().get(url).query(payload).send().await?;
        println!("URL: {}", response.url());

        Ok(response)
    }

    /// Retrieve the promotion tabs, and the associated run-times.
    async fn promotion_tabs(&self) -> Result<PromotionTabs> {
        let response = self.endpoint_get("/promotion-tabs", &[]).await?;

        Ok(response.json().await?)
    }

    /// Return all the products that are part of the provided promotion
    async fn promotion_group(
        &self,
        tab: &TabId,
        runtime: &RuntimeId,
        store_id: Option<u32>,
        sorted_by: Option<SortedByQuery>,
    ) -> Result<PromotionGroup> {
        let url = format!("/promotion-tabs/{}/{}", tab.as_ref(), runtime.as_ref());
        let response = self
            .endpoint_get(
                &url,
                &[
                    ("store_id", store_id.map(|i| i.to_string()).as_deref()),
                    ("sorted_by", sorted_by.map(|i| format!("{:?}", i)).as_deref()),
                ],
            )
            .await?;

        Ok(response.json().await?)
    }
}
