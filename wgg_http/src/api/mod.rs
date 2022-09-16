use crate::api::auth::{AuthMutation, AuthQuery};
use crate::api::error::GraphqlError;
use crate::api::providers::ProviderQuery;
use crate::config::SharedConfig;
use async_graphql::{EmptySubscription, MergedObject, Schema};
use sea_orm::DatabaseConnection;
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

use crate::api::aggregate_ingredients::AggregateQuery;
pub use auth::{create_user, UserCreateInput};
pub(crate) use ctx::*;
pub(crate) use routes::config;

pub type WggSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;
pub type GraphqlResult<T, E = GraphqlError> = std::result::Result<T, E>;

/// State to be shared between all routes, and available as an ExtensionLayer/Context
#[derive(Clone)]
pub struct State {
    pub(crate) db: DatabaseConnection,
    pub(crate) config: SharedConfig,
    pub(crate) providers: Arc<WggProvider>,
}

#[derive(MergedObject, Default)]
pub struct QueryRoot(ProviderQuery, AuthQuery, AggregateQuery);

#[derive(MergedObject, Default)]
pub struct MutationRoot(AuthMutation);
