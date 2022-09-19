//! Re-export all entity files from the [wgg_db_entity] crate, alongside specific repository methods when needed.
pub type Id = i32;

pub mod agg_ingredients;
pub mod agg_ingredients_links;
pub mod cart;
pub mod cart_contents;
pub mod cart_tally;
pub mod providers;
pub mod users;
pub mod users_tokens;

use async_graphql::async_trait;
use sea_orm::strum::IntoEnumIterator;
use sea_orm::{
    ActiveValue, ColumnTrait, ConnectionTrait, DbErr, DeleteMany, EntityTrait, IntoActiveValue, PaginatorTrait,
    PrimaryKeyToColumn, PrimaryKeyTrait, QueryFilter, QuerySelect, Select, Value,
};

pub trait EntityExt: EntityTrait {
    /// Find all entities which have their primary key in the provided iterator.
    ///
    /// Useful for Dataloader queries.
    ///
    /// # Note
    ///
    /// The default implementation only works for non-composite primary keys.
    fn find_by_ids<T: IntoIterator<Item = <Self::PrimaryKey as PrimaryKeyTrait>::ValueType>>(ids: T) -> Select<Self>
    where
        sea_orm::Value: From<<Self::PrimaryKey as PrimaryKeyTrait>::ValueType>,
    {
        let mut pkeys = Self::PrimaryKey::iter();

        if let Some(key) = pkeys.next() {
            let col = key.into_column();
            Self::find().filter(col.is_in(ids))
        } else {
            panic!("In order to get by ID one needs at least one primary key!")
        }
    }

    /// Delete all entities which have their primary key in the provided iterator.
    ///
    /// # Note
    ///
    /// The default implementation only works for non-composite primary keys.
    fn delete_by_ids<T: IntoIterator<Item = <Self::PrimaryKey as PrimaryKeyTrait>::ValueType>>(
        ids: T,
    ) -> DeleteMany<Self>
    where
        sea_orm::Value: From<<Self::PrimaryKey as PrimaryKeyTrait>::ValueType>,
    {
        let mut pkeys = Self::PrimaryKey::iter();

        if let Some(key) = pkeys.next() {
            let col = key.into_column();
            Self::delete_many().filter(col.is_in(ids))
        } else {
            panic!("In order to get by ID one needs at least one primary key!")
        }
    }
}

// Blanket implementation for everything with an [Id] (non-composite) as primary key
impl<T: EntityTrait> EntityExt for T where <Self::PrimaryKey as PrimaryKeyTrait>::ValueType: From<Id> {}

// Needed to ensure we don't repeat ourselves everywhere...
#[async_trait::async_trait]
pub trait SelectExt<E: EntityTrait> {
    async fn one_or_err<'a, C>(self, db: &C) -> Result<E::Model, DbErr>
    where
        C: ConnectionTrait;

    fn offset_paginate<C>(self, limit: u64, db: &C) -> OffsetPaginator<C, E>
    where
        C: ConnectionTrait;
}

#[async_trait::async_trait]
impl<E: EntityTrait> SelectExt<E> for Select<E> {
    /// Return a single object, or error out with [DbErr::RecordNotFound] if no record exists.
    async fn one_or_err<'a, C>(self, db: &C) -> Result<E::Model, DbErr>
    where
        C: ConnectionTrait,
    {
        self.one(db)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("No record found".to_string()))
    }

    /// Create a offset-based paginator.
    ///
    /// Differs from the default [Self::paginate] in that it allows arbitrary offsets, not just on page boundaries.
    ///
    /// # Arguments
    ///
    /// * `limit` - The maximum amount of items returned in a query.
    fn offset_paginate<C>(self, limit: u64, db: &C) -> OffsetPaginator<C, E>
    where
        C: ConnectionTrait,
    {
        OffsetPaginator { query: self, limit, db }
    }
}

#[derive(Clone, Debug)]
pub struct OffsetPaginator<'db, C, E>
where
    C: ConnectionTrait,
    E: EntityTrait + 'db,
{
    query: Select<E>,
    limit: u64,
    db: &'db C,
}

impl<'db, C: ConnectionTrait, E: EntityTrait> OffsetPaginator<'db, C, E> {
    /// Fetch all models, with the existing limit and provided limit.
    pub async fn fetch_offset(&self, offset: u64) -> Result<Vec<E::Model>, DbErr> {
        let results = self.query.clone().limit(self.limit).offset(offset).all(self.db).await?;

        Ok(results)
    }

    /// Return the total amount of items for this query.
    pub async fn num_items(&self) -> Result<u64, DbErr>
    where
        E::Model: Sync,
    {
        let query = self.query.clone();

        let results = PaginatorTrait::count(query, self.db).await?;

        Ok(results as u64)
    }

    /// Performs both [Self::fetch_offset] and [Self::num_items] concurrently, and returns the results.
    pub async fn fetch_and_count(&self, offset: u64) -> Result<(Vec<E::Model>, u64), DbErr>
    where
        E::Model: Sync,
    {
        let (items, count) = futures::future::join(self.fetch_offset(offset), self.num_items()).await;

        Ok((items?, count?))
    }
}

pub trait IntoActiveValueExt<V: Into<Value>> {
    /// The default `into_active_value` converts an `Option<T> -> ActiveValue<Option<T>>`.
    ///
    /// This is undesired for our use-case, where we frequently have optional, updates for non-nullable values (aka, single `Option`)
    ///
    /// There is probably an existing trait/method which does what we want, but it has yet to be discovered.
    fn into_flattened_active_value(self) -> ActiveValue<V>;
}

impl<T: IntoActiveValue<T> + Into<Value>> IntoActiveValueExt<T> for Option<T> {
    fn into_flattened_active_value(self) -> ActiveValue<T> {
        if let Some(value) = self {
            ActiveValue::Set(value)
        } else {
            ActiveValue::NotSet
        }
    }
}
