use crate::api::error::GraphqlError;
use crate::api::{GraphqlResult, State};
use crate::db;
use crate::db::Id;
use async_graphql::async_trait;
use axum::extract::{FromRequest, RequestParts};
use cookie::Key;
use sea_orm::DatabaseConnection;
use tower_cookies::{Cookies, PrivateCookies};

static SESSION_KEY: &str = "session_key";

pub type HashedPassword = String;
pub type SessionToken = String;

/// Represents a user that is already logged in.
/// Is an `extractor` and can therefore be requested in HTTP service methods.
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, async_graphql::SimpleObject)]
pub struct AuthContext {
    pub id: Id,
    pub email: String,
    pub username: String,
}

#[async_trait::async_trait]
impl<B: Send> FromRequest<B> for AuthContext {
    type Rejection = GraphqlError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let extensions = req.extensions();

        let cookies = extensions
            .get::<Cookies>()
            .cloned()
            .ok_or_else(|| GraphqlError::InternalError("Cookie extract failure".to_string()))?;

        let key = extensions
            .get::<Key>()
            .cloned()
            .ok_or_else(|| GraphqlError::InternalError("Key extract failure".to_string()))?;

        let cookies = RepubCookies::from_cookies(cookies, &key);
        let state = extensions
            .get::<State>()
            .ok_or_else(|| GraphqlError::InternalError("DB extract failure".to_string()))?;

        check_login_status(&state.db, cookies).await
    }
}

pub async fn check_login_status(db: &DatabaseConnection, cookies: RepubCookies<'_>) -> GraphqlResult<AuthContext> {
    if let Some(session_token) = cookies.cookies.get(SESSION_KEY) {
        let user = db::users::find_user_by_token(session_token.value())
            .one(db)
            .await?
            .ok_or(GraphqlError::Unauthorized)?;

        Ok(AuthContext {
            id: user.id,
            email: user.email,
            username: user.username,
        })
    } else {
        Err(GraphqlError::Unauthorized)
    }
}

pub struct RepubCookies<'a> {
    pub cookies: PrivateCookies<'a>,
}

impl RepubCookies<'_> {
    pub fn from_cookies(cookies: Cookies, key: &Key) -> RepubCookies<'_> {
        RepubCookies {
            cookies: cookies.private(key),
        }
    }
}
