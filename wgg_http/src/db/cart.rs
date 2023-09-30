use sea_orm::sea_query::IntoCondition;
use sea_orm::{ColumnTrait, Condition, ConnectionTrait, DbErr, EntityTrait, QueryFilter};
pub use wgg_db_entity::cart::*;
use wgg_db_entity::{DbId, SelectExt};

pub async fn get_active_cart_for_user(user_id: DbId, db: &impl ConnectionTrait) -> Result<Model, DbErr> {
    Entity::find()
        .filter(has_user(user_id))
        .filter(is_completed().not())
        .one_or_err(db)
        .await
}

pub fn is_cart_or_active_cart(cart_id: Option<DbId>, user_id: DbId) -> Condition {
    if let Some(cart_id) = cart_id {
        has_user(user_id).add(has_id(cart_id))
    } else {
        is_active_for_user(user_id)
    }
    .into_condition()
}

pub fn is_active_for_user(user_id: DbId) -> Condition {
    has_user(user_id).add(is_completed().not())
}

pub fn has_id(cart_id: DbId) -> Condition {
    Column::Id.eq(cart_id).into_condition()
}

pub fn has_user(user_id: DbId) -> Condition {
    Column::UserId.eq(user_id).into_condition()
}

pub fn is_completed() -> Condition {
    Column::CompletedAt.is_not_null().into_condition()
}
