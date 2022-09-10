use crate::api::auth::mutation::UserCreateInput;
use crate::api::error::GraphqlError;
use crate::api::{GraphqlResult, State};
use crate::db;
use crate::db::Id;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use async_graphql::async_trait;
use axum::extract::{FromRequest, RequestParts};
use cookie::Key;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, IntoActiveValue};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait};
use tower_cookies::{Cookies, PrivateCookies};
use wgg_db_entity::users::Model;

static SESSION_KEY: &str = "session_key";

mod mutation;
mod query;

pub use mutation::AuthMutation;
pub use query::AuthQuery;

/// Represents a user that is already logged in.
/// Implements [axum::extract::FromRequest] and can therefore be requested in HTTP service methods.
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, async_graphql::SimpleObject)]
pub struct AuthContext {
    pub id: Id,
    pub email: String,
    pub username: String,
    pub is_admin: bool,
}

#[derive(Debug, Clone, async_graphql::InputObject)]
pub struct LoginInput {
    /// The email of the user account
    pub email: String,
    /// The account's password
    pub password: String,
}

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

    tracing::trace!("Login success");

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

        if let Some(session_token) = cookies.cookies.get(SESSION_KEY) {
            verify_login_status(&state.db, session_token.value()).await
        } else {
            Err(GraphqlError::Unauthorized)
        }
    }
}

/// Verify whether the provided session token is still valid, and if so, returns the
async fn verify_login_status(db: &DatabaseConnection, session_token: &str) -> GraphqlResult<AuthContext> {
    let user = db::users::find_user_by_token(session_token)
        .one(db)
        .await?
        .ok_or(GraphqlError::Unauthorized)?;

    Ok(user.into())
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

impl From<db::users::Model> for AuthContext {
    fn from(model: Model) -> Self {
        AuthContext {
            id: model.id,
            email: model.email,
            username: model.username,
            is_admin: model.is_admin,
        }
    }
}
