//! Re-export all entity files from the [wgg_db_entity] crate, alongside specific repository methods when needed.
pub type Id = i32;

pub mod agg_ingredients;
pub mod agg_ingredients_links;
pub mod providers;
pub mod users;
pub mod users_tokens;

use sea_orm::strum::IntoEnumIterator;
use sea_orm::{ColumnTrait, EntityTrait, PrimaryKeyToColumn, PrimaryKeyTrait, QueryFilter, Select};

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
