use crate::db::Id;
use sea_orm::sea_query::IntoCondition;
use sea_orm::{ColumnTrait, Condition};
pub use wgg_db_entity::agg_ingredients::*;

/// Condition for selecting entities with the provided name.
pub fn has_name_like(name: &str) -> Condition {
    Column::Name.like(name).into_condition()
}

pub fn created_by(user_id: Id) -> Condition {
    Column::CreatedBy.eq(user_id).into_condition()
}
