use crate::api::auth::AuthContext;
use crate::api::error::GraphqlError;
use crate::config::SharedConfig;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{EmptyMutation, EmptySubscription, MergedObject, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Router};
use sea_orm::DatabaseConnection;

mod auth;
mod ctx;
pub(crate) mod dataloader;
mod error;

pub type WggSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;
pub type GraphqlResult<T, E = GraphqlError> = std::result::Result<T, E>;

/// State to be shared between all routes, and available as an ExtensionLayer/Context
#[derive(Clone)]
pub struct State {
    pub(crate) db: DatabaseConnection,
    pub(crate) config: SharedConfig,
}

#[derive(MergedObject, Default)]
pub struct QueryRoot;

#[derive(MergedObject, Default)]
pub struct MutationRoot(EmptyMutation);

pub fn config(schema: WggSchema) -> Router {
    Router::new().nest(
        "/graphql",
        axum::Router::new()
            .route("/", get(index_playground).post(index))
            .route("/ws", GraphQLSubscription::new(schema.clone())),
    )
}

async fn index(schema: Extension<WggSchema>, req: GraphQLRequest, user: Option<AuthContext>) -> GraphQLResponse {
    schema.execute(req.0.data(user)).await.into()
}

async fn index_playground(_: Option<AuthContext>) -> GraphqlResult<impl IntoResponse> {
    Ok(axum::response::Html(playground_source(
        GraphQLPlaygroundConfig::new("/api/graphql").subscription_endpoint("/api/graphql/ws"),
    )))
}
