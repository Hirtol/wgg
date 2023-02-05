use crate::api::auth::AuthContext;
use crate::api::{GraphqlResult, WggSchema};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Router};

/// Root config for all GraphQL queries
pub fn config(schema: WggSchema) -> Router {
    Router::new().nest(
        "/graphql",
        axum::Router::new()
            .route("/", get(index_playground).post(index))
            .route_service("/ws", GraphQLSubscription::new(schema)),
    )
}

#[tracing::instrument(skip(schema, cookies, req))]
async fn index(
    schema: Extension<WggSchema>,
    cookies: tower_cookies::Cookies,
    user: Option<AuthContext>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let req = req.0.data(cookies).data(user);

    schema.execute(req).await.into()
}

#[tracing::instrument(skip_all)]
async fn index_playground(_: Option<AuthContext>) -> GraphqlResult<impl IntoResponse> {
    Ok(axum::response::Html(playground_source(
        GraphQLPlaygroundConfig::new("/api/graphql").subscription_endpoint("/api/graphql/ws"),
    )))
}
