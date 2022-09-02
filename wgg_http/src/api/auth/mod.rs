use crate::api::error::GraphqlError;
use crate::api::{GraphqlResult, State};
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

pub async fn check_login_status(db: &DatabaseConnection, cookies: RepubCookies<'_>) -> GraphqlResult<AuthContext> {
    if let Some(session_token) = cookies.cookies.get(SESSION_KEY) {
        // let mut connection = db
        //     .reader()
        //     .acquire()
        //     .await
        //     .map_err(|_| ApiError::InternalError("DB Error".to_string()))?;
        // let user = UserRepository::get_user_from_token(&mut *connection, session_token.value())
        //     .await
        //     .map_err(|_| ApiError::Unauthorized("Need login".to_string()))?;
        //
        // let result = UserRepository::get_entity_with_permissions(&mut *connection, user)
        //     .await
        //     .map_err(|_| ApiError::InternalError("Permission Error".to_string()))?;

        // return Ok(result.into());
        Ok(AuthContext {
            id: 0,
            email: "".to_string(),
            username: "".to_string(),
        })
    } else {
        Err(GraphqlError::Unauthorized)
    }
}
