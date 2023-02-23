use crate::api::aggregate_ingredients::{AggregateMutation, AggregateQuery};
use crate::api::auth::{AuthMutation, AuthQuery};
use crate::api::cart::{CartMutation, CartQuery};
use crate::api::error::GraphqlError;
use crate::api::providers::ProviderQuery;
use crate::config::SharedConfig;
use crate::db::Id;
use async_graphql::extensions::{Extension, ExtensionContext, ExtensionFactory, NextExecute};
use async_graphql::{async_trait, EmptySubscription, MergedObject, Response, Schema};
use sea_orm::DatabaseConnection;
use std::collections::BTreeMap;
use std::sync::Arc;
use wgg_providers::models::Provider;
use wgg_providers::WggProvider;
use wgg_scheduler::JobScheduler;

mod aggregate_ingredients;
mod auth;
mod cart;

mod ctx;
pub(crate) mod dataloader;
mod error;
mod macros;
mod pagination;
mod providers;
mod routes;
pub mod scheduled_jobs;

pub use auth::{create_user, UserCreateInput};
pub(crate) use ctx::*;
pub(crate) use routes::config;

pub type WggSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;
pub type GraphqlResult<T, E = GraphqlError> = std::result::Result<T, E>;
/// A product id for an arbitrary provider.
type ProductId = String;

/// The maximum amount of items that should be allowed to be deleted within a single request.
const MAX_AMOUNT_DELETE: usize = 20;

/// State to be shared between all routes, and available as an ExtensionLayer/Context
#[derive(Clone)]
pub struct AppState {
    pub(crate) db: DatabaseConnection,
    pub(crate) config: SharedConfig,
    pub(crate) providers: Arc<WggProvider>,
    pub(crate) scheduler: JobScheduler,
    /// Lists all providers available in the database and their associated Ids.
    ///
    /// This assumes no external modification of the database *whilst* the application is running!
    pub(crate) db_providers: BTreeMap<Provider, Id>,
}

impl AppState {
    /// Quickly find the [Provider] associated with the given `id`.
    pub fn provider_from_id(&self, id: Id) -> Provider {
        self.db_providers
            .iter()
            .find(|&(_, db_id)| *db_id == id)
            .map(|item| *item.0)
            .expect("Expected a new provider to exist in the database when it doesn't!")
    }

    /// Quickly find the `Id` for the given `provider`.
    pub fn provider_id_from_provider(&self, provider: &Provider) -> Id {
        self.db_providers
            .get(provider)
            .copied()
            .expect("Expected a new provider to exist in the database when it doesn't!")
    }
}

#[derive(MergedObject, Default)]
pub struct QueryRoot(ProviderQuery, AuthQuery, AggregateQuery, CartQuery);

#[derive(MergedObject, Default)]
pub struct MutationRoot(AuthMutation, AggregateMutation, CartMutation);

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
            let errors = result
                .errors
                .iter()
                .map(|r| TraceErrorLog {
                    source: &r.source,
                    locations: &r.locations,
                    path: &r.path,
                })
                .collect::<Vec<_>>();
            tracing::warn!(?errors, "Error occurred in GraphQL resolution");
        }

        result
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct TraceErrorLog<'a> {
    /// The original error
    pub source: &'a Option<Arc<dyn std::error::Error + Send + Sync>>,
    /// Where the error occurred.
    pub locations: &'a Vec<async_graphql::Pos>,
    /// If the error occurred in a resolver, the path to the error.
    pub path: &'a Vec<async_graphql::PathSegment>,
}
