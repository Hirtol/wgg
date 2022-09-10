use crate::db;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect, QueryTrait, Select};
pub use wgg_db_entity::users::*;

pub fn find_user_by_token(token: &str) -> Select<Entity> {
    Entity::find().filter(
        Column::Id.in_subquery(
            db::users_tokens::find_by_token(token)
                .filter(db::users_tokens::non_expired())
                .select_only()
                .column(db::users_tokens::Column::Id)
                .into_query(),
        ),
    )
}
