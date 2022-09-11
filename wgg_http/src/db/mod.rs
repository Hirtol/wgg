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
    ActiveValue, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, IntoActiveValue, ModelTrait, PrimaryKeyToColumn,
    PrimaryKeyTrait, QueryFilter, Select, Value,
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
}

// Blanket implementation for everything with an [Id] as primary key
impl<T: EntityTrait> EntityExt for T where <Self::PrimaryKey as PrimaryKeyTrait>::ValueType: From<Id> {}

// Needed to ensure we don't repeat ourselves everywhere...
#[async_trait::async_trait]
pub trait SelectExt {
    type Model: ModelTrait;
    async fn one_or_err<'a, C>(self, db: &C) -> Result<Self::Model, DbErr>
    where
        C: ConnectionTrait;
}

#[async_trait::async_trait]
impl<T: EntityTrait> SelectExt for Select<T> {
    type Model = T::Model;

    async fn one_or_err<'a, C>(self, db: &C) -> Result<Self::Model, DbErr>
    where
        C: ConnectionTrait,
    {
        self.one(db)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("No record found".to_string()))
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
