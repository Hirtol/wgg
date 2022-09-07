use crate::api::ctx::ContextExt;
use crate::api::GraphqlResult;
use async_graphql::*;
use wgg_providers::models::{Autocomplete, Product, PromotionCategory, Provider, SearchProduct};

#[derive(Default)]
pub struct SearchQuery;

#[Object]
impl SearchQuery {
    #[tracing::instrument(skip(self, ctx))]
    async fn autocomplete(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The current user input")] query: String,
    ) -> GraphqlResult<Vec<Autocomplete>> {
        let state = ctx.wgg_state();
        let response = state.providers.autocomplete(Provider::Jumbo, query).await?;

        Ok(response)
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn search(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The product query")] query: String,
    ) -> GraphqlResult<Vec<SearchProduct>> {
        let state = ctx.wgg_state();
        let response = state.providers.search_all(query).await?;

        Ok(response.items)
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn product(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The product vendor/provider")] provider: Provider,
        #[graphql(desc = "The product id")] product_id: String,
    ) -> GraphqlResult<Product> {
        let state = ctx.wgg_state();
        let response = state.providers.product(provider, product_id).await?;

        Ok(response)
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn promotions(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The product vendor/provider")] provider: Provider,
    ) -> GraphqlResult<Vec<PromotionCategory>> {
        let state = ctx.wgg_state();
        let response = state.providers.promotions(provider).await?;

        Ok(response)
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn promotions_sublist(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The product vendor/provider")] provider: Provider,
        #[graphql(desc = "The sublist id")] sublist_id: String,
    ) -> GraphqlResult<Vec<SearchProduct>> {
        let state = ctx.wgg_state();
        let response = state.providers.promotions_sublist(provider, sublist_id).await?;

        Ok(response.items)
    }
}
