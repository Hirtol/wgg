use crate::api::{ContextExt, GraphqlResult};
use async_graphql::{Context, Object};
use wgg_providers::models::{Provider, ProviderInfo, WggAutocomplete, WggProduct, WggSaleCategory, WggSearchProduct};

#[derive(Default)]
pub struct ProviderQuery;

#[Object]
impl ProviderQuery {
    #[tracing::instrument(skip(self, ctx))]
    async fn pro_autocomplete(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The product vendor/provider", default_with = "Provider::Picnic")] provider: Provider,
        #[graphql(desc = "The current user input")] query: String,
    ) -> GraphqlResult<Vec<WggAutocomplete>> {
        let state = ctx.wgg_state();
        let response = state.providers.autocomplete(provider, query).await?;

        Ok(response)
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn pro_search(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The product vendor/provider", default_with = "Provider::Picnic")] provider: Provider,
        #[graphql(desc = "The product query")] query: String,
    ) -> GraphqlResult<Vec<WggSearchProduct>> {
        let state = ctx.wgg_state();
        let response = state.providers.search(provider, query, None).await?;

        Ok(response.items)
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn pro_search_all(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The product query")] query: String,
    ) -> GraphqlResult<Vec<WggSearchProduct>> {
        let state = ctx.wgg_state();
        let response = state.providers.search_all(query).await?;

        Ok(response.items)
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn pro_product(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The product vendor/provider", default_with = "Provider::Picnic")] provider: Provider,
        #[graphql(desc = "The product id")] product_id: String,
    ) -> GraphqlResult<WggProduct> {
        let state = ctx.wgg_state();
        let response = state.providers.product(provider, product_id).await?;

        Ok(response)
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn pro_promotions(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The product vendor/provider", default_with = "Provider::Picnic")] provider: Provider,
    ) -> GraphqlResult<Vec<WggSaleCategory>> {
        let state = ctx.wgg_state();
        let response = state.providers.promotions(provider).await?;

        Ok(response)
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn pro_promotions_all(&self, ctx: &Context<'_>) -> GraphqlResult<Vec<WggSaleCategory>> {
        let state = ctx.wgg_state();
        let response = state.providers.promotions_all().await?;

        Ok(response)
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn pro_promotions_sublist(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The product vendor/provider", default_with = "Provider::Picnic")] provider: Provider,
        #[graphql(desc = "The sublist id")] sublist_id: String,
    ) -> GraphqlResult<Vec<WggSearchProduct>> {
        let state = ctx.wgg_state();
        let response = state.providers.promotions_sublist(provider, sublist_id).await?;

        Ok(response.items)
    }

    /// Return all providers which are currently active for this server.
    #[tracing::instrument(skip(self, ctx))]
    async fn pro_providers(&self, ctx: &Context<'_>) -> Vec<ProviderInfo> {
        let state = ctx.wgg_state();

        state
            .providers
            .iter()
            .map(|prov| ProviderInfo {
                provider: prov.provider(),
                logo_url: prov.logo_url(),
            })
            .collect()
    }
}
