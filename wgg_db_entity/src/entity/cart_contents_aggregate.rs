//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.3

use sea_orm::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "cart_contents_aggregate"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Eq)]
pub struct Model {
    pub id: i32,
    pub cart_id: i32,
    pub aggregate_id: i32,
    pub quantity: i32,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    CartId,
    AggregateId,
    Quantity,
    CreatedAt,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    Id,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = i32;
    fn auto_increment() -> bool {
        false
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    AggIngredients,
    Cart,
}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
            Self::Id => ColumnType::Integer.def(),
            Self::CartId => ColumnType::Integer.def(),
            Self::AggregateId => ColumnType::Integer.def(),
            Self::Quantity => ColumnType::Integer.def(),
            Self::CreatedAt => ColumnType::Timestamp.def(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::AggIngredients => Entity::belongs_to(super::agg_ingredients::Entity)
                .from(Column::AggregateId)
                .to(super::agg_ingredients::Column::Id)
                .into(),
            Self::Cart => Entity::belongs_to(super::cart::Entity)
                .from(Column::CartId)
                .to(super::cart::Column::Id)
                .into(),
        }
    }
}

impl Related<super::agg_ingredients::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AggIngredients.def()
    }
}

impl Related<super::cart::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Cart.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
