use crate::db::Id;
use sea_orm::sea_query::IntoCondition;
use sea_orm::{ColumnTrait, Condition};
pub use wgg_db_entity::cart::*;

pub fn has_user(user_id: Id) -> Condition {
    Column::Id.eq(user_id).into_condition()
}

pub fn is_completed() -> Condition {
    Column::CompletedAt.is_not_null().into_condition()
}
