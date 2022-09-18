use crate::db::Id;
use async_graphql::EnumType;
use sea_orm::{ConnectionTrait, EntityTrait};
use std::collections::BTreeMap;
pub use wgg_db_entity::providers::*;
use wgg_providers::models::Provider;

pub fn all_providers() -> impl Iterator<Item = String> {
    Provider::items().iter().map(|i| i.name.to_string())
}

pub async fn all_db_providers(db: &impl ConnectionTrait) -> anyhow::Result<BTreeMap<Provider, Id>> {
    let entities = Entity::find().all(db).await?;

    entities
        .into_iter()
        .map(|item| Ok((item.name.parse()?, item.id)))
        .collect()
}
