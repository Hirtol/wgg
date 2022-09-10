use sea_orm::sea_query::IntoCondition;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Select};
pub use wgg_db_entity::users_tokens::*;

pub fn find_by_token(token: &str) -> Select<Entity> {
    Entity::find().filter(Column::Token.eq(token))
}

/// Condition for selecting entities which have not yet expired
pub fn non_expired() -> impl IntoCondition {
    Column::Expires.gt(chrono::Local::now().naive_utc())
}
