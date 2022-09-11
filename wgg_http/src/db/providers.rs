use async_graphql::EnumType;
pub use wgg_db_entity::providers::*;
use wgg_providers::models::Provider;

pub fn all_providers() -> impl Iterator<Item = String> {
    Provider::items().iter().map(|i| i.name.to_string())
}
