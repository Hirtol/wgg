use crate::db;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect, QueryTrait, Select};
pub use wgg_db_entity::users::*;

/// Finds the user associated with the provided *valid* & *unexpired* token.
///
/// If the token is expired this request will return nothing.
pub fn find_user_by_token(token: &str) -> Select<Entity> {
    Entity::find().filter(
        Column::Id.in_subquery(
            db::users_tokens::find_by_token(token)
                .filter(db::users_tokens::non_expired())
                .select_only()
                .column(db::users_tokens::Column::UserId)
                .into_query(),
        ),
    )
}
