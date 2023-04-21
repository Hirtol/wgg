use crate::db;
use itertools::Itertools;
use rand::distributions::Alphanumeric;
use rand::Rng;
use sea_orm::sea_query::IntoCondition;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Condition, ConnectionTrait, EntityTrait, IntoActiveValue, QueryFilter,
    Select,
};
pub use wgg_db_entity::users_tokens::*;

/// Create a new session token for the given user.
#[tracing::instrument(skip_all, fields(id=%user.id, username=%user.username, email=%user.email))]
pub async fn create_session_token(db: &impl ConnectionTrait, user: &db::users::Model) -> anyhow::Result<Model> {
    let items = std::iter::repeat(())
        .map(|()| rand::rngs::OsRng.sample(Alphanumeric))
        .take(64)
        .collect_vec();

    let token = String::from_utf8(items)?;

    let active_model = ActiveModel {
        user_id: ActiveValue::Set(user.id),
        token: token.into_active_value(),
        expires: ActiveValue::Set(chrono::Utc::now() + chrono::Duration::weeks(4)),
        ..Default::default()
    };

    let result = active_model.insert(db).await?;

    Ok(result)
}

/// Create a select for finding a User Token from any `token`.
pub fn find_by_token(token: &str) -> Select<Entity> {
    Entity::find().filter(has_token(token))
}

/// Condition for selecting entities with the provided token.
pub fn has_token(token: &str) -> Condition {
    Column::Token.eq(token).into_condition()
}

/// Condition for selecting entities which have not yet expired
pub fn non_expired() -> Condition {
    Column::Expires.gt(chrono::Utc::now()).into_condition()
}
