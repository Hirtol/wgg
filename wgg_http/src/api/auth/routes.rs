use crate::api::auth::mutation::LoginInput;
use crate::api::auth::AuthContext;
use crate::api::{AppState, GraphqlResult};
use axum::extract::State;
use axum::routing::{get, MethodFilter};
use axum::{Extension, Json, Router};
use cookie::Key;

pub fn config() -> Router<AppState> {
    Router::new().nest(
        "/auth",
        Router::new()
            .route("/", get(current_user))
            .route("/login", axum::routing::on(MethodFilter::POST, login)),
    )
}

pub async fn current_user(user: AuthContext) -> Json<AuthContext> {
    Json(user)
}

pub async fn login(
    State(state): State<AppState>,
    cookies: tower_cookies::Cookies,
    key: Extension<Key>,
    input: Json<LoginInput>,
) -> GraphqlResult<Json<AuthContext>> {
    todo!()
}
