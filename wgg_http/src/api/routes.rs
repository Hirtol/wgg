use crate::api::auth::AuthContext;
use crate::api::error::GraphqlError;
use crate::api::{GraphqlResult, WggSchema};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Router};

/// Root config for all GraphQL queries
pub fn config(schema: WggSchema) -> Router<super::AppState> {
    Router::new().merge(super::auth::config()).nest(
        "/graphql",
        Router::new()
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
async fn index_playground(
    _: Option<AuthContext>,
    State(state): State<super::AppState>,
) -> GraphqlResult<impl IntoResponse> {
    if state.config.load().app.graphql_playground {
        Ok(axum::response::Html(playground_source(
            GraphQLPlaygroundConfig::new("/api/graphql")
                .with_setting("request.credentials", "same-origin")
                .subscription_endpoint("/api/graphql/ws"),
        )))
    } else {
        Err(GraphqlError::PlaygroundDisabled)
    }
}
