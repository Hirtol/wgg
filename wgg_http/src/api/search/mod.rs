use crate::api::error::GraphqlError;
use crate::api::GraphqlResult;
use async_graphql::*;
use wgg_providers::models::{Autocomplete, SearchItem};

#[derive(Default)]
pub struct SearchQuery;

#[Object]
impl SearchQuery {
    async fn autocomplete(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The current user input")] query: String,
    ) -> GraphqlResult<Vec<SearchItem>> {
        Err(GraphqlError::Unauthorized)
    }
}
