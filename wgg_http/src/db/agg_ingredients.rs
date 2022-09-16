use sea_orm::sea_query::{ConditionExpression};
use sea_orm::ColumnTrait;
pub use wgg_db_entity::agg_ingredients::*;

/// Condition for selecting entities with the provided name.
pub fn has_name_like(name: &str) -> impl Into<ConditionExpression> {
    Column::Name.like(name)
}
