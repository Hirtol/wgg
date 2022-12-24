mod mutation;
mod objects;
mod query;
mod service;

pub use query::AggregateQuery;
pub use mutation::AggregateMutation;
pub use objects::AggregateIngredient;
pub use service::get_associated_aggregate_for_product;
