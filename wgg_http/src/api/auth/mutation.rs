use crate::api::auth::AuthContext;
use crate::api::ctx::ContextExt;
use crate::api::error::GraphqlError;
use crate::api::GraphqlResult;
use crate::db;
use crate::db::{Id, IntoActiveValueExt, SelectExt};
use async_graphql::{Context, Object};
use cookie::time::OffsetDateTime;
use cookie::{Cookie, SameSite};

use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, QueryFilter, TransactionTrait};

#[derive(Default)]
pub struct AuthMutation;

#[Object]
impl AuthMutation {
    /// Create a new user.
    ///
    /// # Returns
    ///
    /// The newly created user.
    ///
    /// # Accessible By
    ///
    /// Admins.
    async fn user_create(&self, ctx: &Context<'_>, input: UserCreateInput) -> GraphqlResult<UserCreatePayload> {
        let state = ctx.wgg_state();
        let current_user = ctx.wgg_user()?;

        if !current_user.is_admin {
            return Err(GraphqlError::Unauthorized);
        }

        let created_user = super::create_user(&state.db, input).await?;

        Ok(UserCreatePayload { user: created_user })
    }

    /// Update an existing user.
    ///
    /// # Returns
    ///
    /// The updated user.
    ///
    /// # Accessible By
    ///
    /// Admins, or users modifying themselves.
    async fn user_update(
        &self,
        ctx: &Context<'_>,
        id: Id,
        input: UserUpdateChangeSet,
    ) -> GraphqlResult<UserUpdatePayload> {
        let state = ctx.wgg_state();
        let current_user = ctx.wgg_user()?;
        // If the user is not an admin they are still allowed to modify themselves, just not a different user's account.
        if !current_user.is_admin && id != current_user.id {
            return Err(GraphqlError::Unauthorized);
        }

        let tx = state.db.begin().await?;

        let mut to_change = db::users::Entity::find_by_id(id)
            .one_or_err(&tx)
            .await?
            .into_active_model();

        to_change.username = input.username.into_flattened_active_value();
        to_change.email = input.email.into_flattened_active_value();

        let result = to_change.update(&tx).await?;

        tx.commit().await?;

        Ok(UserUpdatePayload { user: result.into() })
    }

    /// Deletes an existing user.
    ///
    /// # Accessible By
    ///
    /// Admins.
    async fn user_delete(&self, ctx: &Context<'_>, id: Id) -> GraphqlResult<UserDeletePayload> {
        let state = ctx.wgg_state();
        let current_user = ctx.wgg_user()?;

        if !current_user.is_admin {
            return Err(GraphqlError::Unauthorized);
        }

        let _ = db::users::Entity::delete_by_id(id).exec(&state.db).await?;

        Ok(UserDeletePayload { id })
    }

    /// Attempt to log in as the provided user
    ///
    /// # Accesible By
    ///
    /// Everyone (also unauthenticated users)
    async fn login(&self, ctx: &Context<'_>, input: LoginInput) -> GraphqlResult<UserLoginPayload> {
        let state = ctx.wgg_state();
        let cookies = ctx.wgg_cookies();

        let (user, session_token) = super::login_user(&state.db, &input).await?;

        let mut cookie = tower_cookies::Cookie::new(super::SESSION_KEY, session_token.token);
        let expiry = cookie::Expiration::DateTime(
            OffsetDateTime::from_unix_timestamp(session_token.expires.timestamp()).unwrap(),
        );

        cookie.set_http_only(true);
        cookie.set_path("/");
        cookie.set_expires(expiry);
        cookie.set_same_site(Some(SameSite::Lax));
        cookie.set_secure(false);

        cookies.cookies.add(cookie);

        Ok(UserLoginPayload { user })
    }

    /// Log out with the current account
    async fn logout(&self, ctx: &Context<'_>) -> GraphqlResult<Id> {
        let state = ctx.wgg_state();
        let cookies = ctx.wgg_cookies();

        let session_token = cookies
            .cookies
            .get(super::SESSION_KEY)
            .ok_or(GraphqlError::ResourceNotFound)?;

        cookies.cookies.remove(Cookie::named(super::SESSION_KEY));

        let _ = db::users_tokens::Entity::delete_many()
            .filter(db::users_tokens::has_token(session_token.value()))
            .exec(&state.db)
            .await?;

        Ok(1)
    }
}

#[derive(Debug, Clone, async_graphql::InputObject)]
pub struct UserCreateInput {
    pub username: String,
    /// The email of the user account
    pub email: String,
    /// The account's password
    pub password: String,
    pub is_admin: bool,
}

#[derive(async_graphql::SimpleObject)]
pub struct UserCreatePayload {
    /// The newly created user.
    pub user: AuthContext,
}

#[derive(async_graphql::SimpleObject)]
pub struct UserDeletePayload {
    /// The Id of the deleted user
    pub id: Id,
}

#[derive(async_graphql::InputObject, Debug)]
pub struct UserUpdateChangeSet {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}

#[derive(async_graphql::SimpleObject)]
pub struct UserUpdatePayload {
    /// The newly updated user.
    pub user: AuthContext,
}

#[derive(async_graphql::SimpleObject)]
pub struct UserLoginPayload {
    /// The newly logged-in user.
    pub user: AuthContext,
}


#[derive(Debug, Clone, async_graphql::InputObject)]
pub struct LoginInput {
    /// The email of the user account
    pub email: String,
    /// The account's password
    pub password: String,
}
