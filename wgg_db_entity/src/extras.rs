//! Extra definitions which should not be touched by auto-generation. Be careful as these need to be updated manually!

mod cart_contents_aggregate {
    use crate::entity;
    use crate::entity::cart_contents_aggregate::*;
    use sea_orm::{EntityTrait, Related, RelationDef};

    impl Related<entity::agg_ingredients_links::Entity> for Entity {
        fn to() -> RelationDef {
            Entity::belongs_to(entity::agg_ingredients_links::Entity)
                .from(Column::AggregateId)
                .to(entity::agg_ingredients_links::Column::AggregateId)
                .into()
        }
    }
}
