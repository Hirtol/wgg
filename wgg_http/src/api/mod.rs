use crate::api::aggregate_ingredients::{AggregateMutation, AggregateQuery};
use crate::api::auth::{AuthMutation, AuthQuery};
use crate::api::error::GraphqlError;
use crate::api::providers::ProviderQuery;
use crate::config::SharedConfig;
use async_graphql::extensions::{Extension, ExtensionContext, ExtensionFactory, NextExecute};
use async_graphql::{async_trait, EmptySubscription, MergedObject, Response, Schema};
use sea_orm::DatabaseConnection;
use std::collections::BTreeMap;
use std::sync::Arc;
use wgg_providers::WggProvider;

mod aggregate_ingredients;
mod auth;
mod ctx;
pub(crate) mod dataloader;
mod error;
mod macros;
mod pagination;
mod providers;
mod routes;
mod search;

use crate::db::Id;
pub use auth::{create_user, UserCreateInput};
pub(crate) use ctx::*;
pub(crate) use routes::config;
use wgg_providers::models::Provider;

pub type WggSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;
pub type GraphqlResult<T, E = GraphqlError> = std::result::Result<T, E>;
/// A product id for an arbitrary provider.
type ProductId = String;

/// State to be shared between all routes, and available as an ExtensionLayer/Context
#[derive(Clone)]
pub struct State {
    pub(crate) db: DatabaseConnection,
    pub(crate) config: SharedConfig,
    pub(crate) providers: Arc<WggProvider>,
    /// Lists all providers available in the database and their associated Ids.
    ///
    /// This assumes no external modification of the database *whilst* the application is running!
    pub(crate) db_providers: BTreeMap<Provider, Id>,
}

#[derive(MergedObject, Default)]
pub struct QueryRoot(ProviderQuery, AuthQuery, AggregateQuery);

#[derive(MergedObject, Default)]
pub struct MutationRoot(AuthMutation, AggregateMutation);

pub struct ErrorTraceExtension;

impl ExtensionFactory for ErrorTraceExtension {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(ErrorTraceExtension)
    }
}

#[async_trait::async_trait]
impl Extension for ErrorTraceExtension {
    async fn execute(
        &self,
        ctx: &ExtensionContext<'_>,
        operation_name: Option<&str>,
        next: NextExecute<'_>,
    ) -> Response {
        let result = next.run(ctx, operation_name).await;

        if result.is_err() {
            tracing::warn!(error=?result.errors, "Error occurred in GraphQL resolution");
        }

        result
    }
}
