use crate::db::{Id, SelectExt};
use sea_orm::sea_query::IntoCondition;
use sea_orm::{ColumnTrait, Condition, ConnectionTrait, DbErr, EntityTrait, QueryFilter};
pub use wgg_db_entity::cart::*;

pub async fn get_active_cart_for_user(user_id: Id, db: &impl ConnectionTrait) -> Result<Model, DbErr> {
    Entity::find()
        .filter(has_user(user_id))
        .filter(is_completed().not())
        .one_or_err(db)
        .await
}

pub fn has_user(user_id: Id) -> Condition {
    Column::UserId.eq(user_id).into_condition()
}

pub fn is_completed() -> Condition {
    Column::CompletedAt.is_not_null().into_condition()
}
