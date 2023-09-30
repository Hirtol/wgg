use sea_orm::sea_query::IntoCondition;
use sea_orm::{ColumnTrait, Condition};
pub use wgg_db_entity::agg_ingredients_links::*;
use wgg_db_entity::DbId;

pub fn related_product(product_id: &str, provider_id: DbId) -> Condition {
    Column::ProviderIngrId
        .eq(product_id)
        .and(Column::ProviderId.eq(provider_id))
        .into_condition()
}

pub fn related_aggregate(aggregate_id: DbId) -> Condition {
    Column::AggregateId.eq(aggregate_id).into_condition()
}
