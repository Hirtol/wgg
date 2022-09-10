use crate::api::auth::AuthContext;
use crate::api::ctx::ContextExt;
use crate::api::error::GraphqlError;
use crate::api::GraphqlResult;
use crate::db;
use crate::db::{Id, IntoActiveValueExt, SelectExt};
use async_graphql::{Context, MaybeUndefined, Object};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, TransactionTrait};

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
}

#[derive(Debug, Clone, async_graphql::InputObject)]
pub struct UserCreateInput {
    pub username: String,
    /// The email of the user account
    pub email: String,
    /// The account's password
    pub password: String,
    #[graphql(skip)]
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
    /// The newly created user.
    pub user: AuthContext,
}
