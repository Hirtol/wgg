use crate::api::auth::mutation::LoginInput;
use crate::api::auth::{AuthContext, WggCookies};
use crate::api::{AppState, GraphqlResult, UserCreateInput};
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, MethodFilter};
use axum::{Extension, Json, Router};
use cookie::Key;

pub fn config() -> Router<AppState> {
    Router::new().nest(
        "/auth",
        Router::new()
            .route("/", get(current_user).post(create_user))
            .route("/login", axum::routing::on(MethodFilter::POST, login))
            .route("/register", axum::routing::on(MethodFilter::POST, register)),
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
    let wgg_cookies = WggCookies::from_cookies(&cookies, &*key);

    let (user, session_token) = super::login_user(&state.db, &input).await?;

    super::insert_auth_cookie(&wgg_cookies, session_token.token, session_token.expires);

    Ok(Json(user))
}

pub async fn register(State(state): State<AppState>, input: Json<RegisterInput>) -> GraphqlResult<StatusCode> {
    todo!("Stub implementation")
}

pub async fn create_user(State(state): State<AppState>, input: Json<UserCreateInput>) -> GraphqlResult<StatusCode> {
    todo!("Stub implementation")
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegisterInput {
    pub username: String,
    /// The email of the user account
    pub email: String,
    /// The account's password
    pub password: String,
}
