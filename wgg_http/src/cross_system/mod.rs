//! Contains data relevant for *both* the DB module and the API module.
mod filter;

pub use filter::{recursive_filter, Filter};
use sea_orm::{ActiveValue, IntoActiveValue, Value};

pub trait IntoActiveValueExtGraphql<V: Into<Value>> {
    /// The default `into_active_value` converts an `Option<T> -> ActiveValue<Option<T>>`.
    ///
    /// This is undesired for our use-case, where we frequently have optional updates for non-nullable values (aka, single `Option`)
    ///
    /// There is probably an existing trait/method which does what we want, but it has yet to be discovered.
    fn into_flattened_active_value(self) -> ActiveValue<V>;
}

impl<T: IntoActiveValue<T> + Into<Value> + sea_orm::sea_query::Nullable> IntoActiveValueExtGraphql<Option<T>>
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

impl<T: IntoActiveValue<T> + Into<Value> + sea_orm::sea_query::Nullable> IntoActiveValueExtGraphql<T>
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
