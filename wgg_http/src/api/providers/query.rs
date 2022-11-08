use crate::api::pagination::{ConnectionResult, QueryResult};
use crate::api::{ContextExt, GraphqlResult};
use async_graphql::{Context, Object};
use wgg_providers::models::{
    Provider, ProviderInfo, WggAutocomplete, WggProduct, WggSaleCategory, WggSaleGroupComplete, WggSearchProduct,
};

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
        after: Option<String>,
        first: Option<i32>,
        #[graphql(desc = "Filters for the collection")] filters: SearchFilter,
    ) -> ConnectionResult<WggSearchProduct> {
        // Assert that the user is logged in.
        let _ = ctx.wgg_user()?;
        let state = ctx.wgg_state();

        crate::api::pagination::offset_query(after, first, |offset, limit| async move {
            let response = state
                .providers
                .search(filters.provider, filters.query, offset.map(|i| i.index() as u32))
                .await?;
            let total_count = response.total_items as u64;

            Ok(QueryResult {
                iter: response.items.into_iter().take(limit),
                total_count,
            })
        })
        .await
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
        after: Option<String>,
        first: Option<i32>,
        #[graphql(desc = "Filters for the collection, defaults to Picnic filter")] filters: Option<PromotionsFilter>,
    ) -> ConnectionResult<WggSaleCategory> {
        // Assert that the user is logged in.
        let _ = ctx.wgg_user()?;
        let state = ctx.wgg_state();
        let filter = filters.unwrap_or(PromotionsFilter {
            provider: Provider::Picnic,
        });

        crate::api::pagination::offset_query(after, first, |offset, limit| async move {
            let response = state.providers.promotions(filter.provider).await?;
            let total_count = response.len() as u64;

            Ok(QueryResult {
                iter: response
                    .into_iter()
                    .skip(offset.unwrap_or_default().index())
                    .take(limit),
                total_count,
            })
        })
        .await
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
    ) -> GraphqlResult<WggSaleGroupComplete> {
        let state = ctx.wgg_state();
        let response = state.providers.promotions_sublist(provider, sublist_id).await?;

        Ok(response)
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

#[derive(Debug, Clone, async_graphql::InputObject)]
struct PromotionsFilter {
    pub provider: Provider,
}

#[derive(Debug, Clone, async_graphql::InputObject)]
struct SearchFilter {
    /// The provider to search in
    pub provider: Provider,
    /// The product name query
    pub query: String,
}
