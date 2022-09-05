use crate::api::ctx::ContextExt;
use crate::api::GraphqlResult;
use async_graphql::*;
use wgg_providers::models::{Autocomplete, Provider, SearchProduct};

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
}
