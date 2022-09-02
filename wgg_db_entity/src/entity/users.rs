//! SeaORM Entity. Generated by sea-orm-codegen 0.9.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    pub email: String,
    pub username: String,
    pub hash: String,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::agg_ingredients::Entity")]
    AggIngredients,
    #[sea_orm(has_many = "super::users_tokens::Entity")]
    UsersTokens,
}

impl Related<super::agg_ingredients::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AggIngredients.def()
    }
}

impl Related<super::users_tokens::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UsersTokens.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
