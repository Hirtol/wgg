//! Contains data relevant for *both* the DB module and the API module.
mod filter;

use crate::db::IntoActiveValueExt;
pub use filter::{recursive_filter, Filter};
use sea_orm::{ActiveValue, IntoActiveValue, Value};

impl<T: IntoActiveValue<T> + Into<Value> + sea_orm::sea_query::Nullable> IntoActiveValueExt<Option<T>>
    for async_graphql::MaybeUndefined<T>
{
    fn into_flattened_active_value(self) -> ActiveValue<Option<T>> {
        use async_graphql::MaybeUndefined;
        match self {
            MaybeUndefined::Undefined => ActiveValue::NotSet,
            MaybeUndefined::Null => ActiveValue::Set(None),
            MaybeUndefined::Value(val) => ActiveValue::Set(Some(val)),
        }
    }
}

impl<T: IntoActiveValue<T> + Into<Value> + sea_orm::sea_query::Nullable> IntoActiveValueExt<T>
    for async_graphql::MaybeUndefined<T>
{
    fn into_flattened_active_value(self) -> ActiveValue<T> {
        use async_graphql::MaybeUndefined;
        match self {
            MaybeUndefined::Value(val) => ActiveValue::Set(val),
            _ => ActiveValue::NotSet,
        }
    }
}
