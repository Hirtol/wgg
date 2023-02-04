use crate::api::error::GraphqlError;
use crate::api::{GraphqlResult, State};
use crate::db;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use async_graphql::async_trait;
use axum::extract::{FromRequestParts};
use axum::http::request::Parts;
use tower_cookies::Key;
use mutation::LoginInput;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, IntoActiveValue};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait};
use tower_cookies::{Cookies, PrivateCookies};

static SESSION_KEY: &str = "session_key";

mod mutation;
mod objects;
mod query;

pub use mutation::{AuthMutation, UserCreateInput};
pub use objects::AuthContext;
pub use query::AuthQuery;

/// Verify the provided login credentials, and if successful, create a new session token in the database.
///
/// Note that the caller should ensure the session token is installed into the client's request.
#[tracing::instrument(skip_all, fields(email=%login.email))]
pub async fn login_user(
    db: &DatabaseConnection,
    login: &LoginInput,
) -> GraphqlResult<(AuthContext, db::users_tokens::Model)> {
    let tx = db.begin().await?;

    let user = db::users::Entity::find()
        .filter(db::users::Column::Email.eq(&*login.email))
        .one(&tx)
        .await?
        .ok_or(GraphqlError::Unauthorized)?;

    verify_password(&login.password, &user.hash)?;

    let session_token = db::users_tokens::create_session_token(&tx, &user).await?;

    tx.commit().await?;

    tracing::debug!(id=%user.id, "Login success");

    Ok((user.into(), session_token))
}

/// Create a new user in the database.
#[tracing::instrument(skip_all, fields(username=%new_user.username, email=%new_user.email))]
pub async fn create_user(db: &impl ConnectionTrait, new_user: UserCreateInput) -> GraphqlResult<AuthContext> {
    let hash_pass = hash_password(&new_user.password)?;

    let new_user = db::users::ActiveModel {
        id: Default::default(),
        email: new_user.email.into_active_value(),
        username: new_user.username.into_active_value(),
        hash: hash_pass.into_active_value(),
        created_at: Default::default(),
        is_admin: new_user.is_admin.into_active_value(),
    };

    let model = new_user.insert(db).await?;

    tracing::debug!(
        user_id = model.id,
        email = model.email,
        username = model.username,
        "New user created"
    );

    Ok(model.into())
}

fn verify_password(password: impl AsRef<[u8]>, hashed_password: impl AsRef<str>) -> GraphqlResult<()> {
    let argon = Argon2::default();
    let password_hash = PasswordHash::new(hashed_password.as_ref())
        .map_err(|e| anyhow::anyhow!("Saved password is no longer valid: {}", e))?;

    argon
        .verify_password(password.as_ref(), &password_hash)
        .map_err(|_| GraphqlError::InvalidInput("Invalid password or login".into()))
}

fn hash_password(password: impl AsRef<[u8]>) -> anyhow::Result<String> {
    let salt = SaltString::generate(rand::rngs::OsRng);
    let argon = Argon2::default();

    Ok(argon
        .hash_password(password.as_ref(), salt.as_ref())
        .map_err(|e| anyhow::anyhow!(e))?
        .to_string())
}

#[async_trait::async_trait]
impl<S: Send + Sync> FromRequestParts<S> for AuthContext {
    type Rejection = GraphqlError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let cookies = Cookies::from_request_parts(parts, state).await.unwrap();
        let extensions = &parts.extensions;

        let key = extensions
            .get::<Key>()
            .cloned()
            .ok_or_else(|| GraphqlError::InternalError("Key extract failure".to_string()))?;

        let cookies = WggCookies::from_cookies(&cookies, &key);
        let state = extensions
            .get::<State>()
            .ok_or_else(|| GraphqlError::InternalError("DB extract failure".to_string()))?;

        if let Some(session_token) = cookies.cookies.get(SESSION_KEY) {
            verify_login_status(&state.db, session_token.value()).await
        } else {
            Err(GraphqlError::Unauthorized)
        }
    }
}

/// Verify whether the provided session token is still valid, and if so, returns the user.
async fn verify_login_status(db: &DatabaseConnection, session_token: &str) -> GraphqlResult<AuthContext> {
    let user = db::users::find_user_by_token(session_token)
        .one(db)
        .await?
        .ok_or(GraphqlError::Unauthorized)?;

    Ok(user.into())
}

pub struct WggCookies<'a> {
    pub cookies: PrivateCookies<'a>,
}

impl WggCookies<'_> {
    pub fn from_cookies<'a>(cookies: &'a Cookies, key: &'a Key) -> WggCookies<'a> {
        WggCookies {
            cookies: cookies.private(key),
        }
    }
}
