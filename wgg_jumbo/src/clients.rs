use crate::models::PromotionTabs;
use crate::Config;
use crate::Result;
use reqwest::Response;

type Query<'a> = [(&'a str, &'a str)];

#[async_trait::async_trait]
pub trait BaseApi {
    fn get_config(&self) -> &Config;

    fn get_http(&self) -> &reqwest::Client;

    #[doc(hidden)]
    #[inline]
    async fn endpoint_get(&self, url_suffix: &str, payload: &Query<'_>) -> Result<Response> {
        let url = self.get_config().get_full_url(url_suffix);
        let response = self.get_http().get(url).query(payload).send().await?;

        Ok(response)
    }

    /// Retrieve the promotion tabs, and the associated run-times.
    async fn promotion_tabs(&self) -> Result<PromotionTabs> {
        let response = self.endpoint_get("/promotion-tabs", &[]).await?;

        Ok(response.json().await?)
    }
}
