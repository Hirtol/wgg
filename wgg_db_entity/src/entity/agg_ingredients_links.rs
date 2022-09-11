//! SeaORM Entity. Generated by sea-orm-codegen 0.9.2

use sea_orm::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "agg_ingredients_links"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
pub struct Model {
    pub id: i32,
    pub aggregate_id: i32,
    pub provider_id: i32,
    pub provider_ingr_id: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    AggregateId,
    ProviderId,
    ProviderIngrId,
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
    Providers,
    AggIngredients,
}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
            Self::Id => ColumnType::Integer.def(),
            Self::AggregateId => ColumnType::Integer.def(),
            Self::ProviderId => ColumnType::Integer.def(),
            Self::ProviderIngrId => ColumnType::String(None).def(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Providers => Entity::belongs_to(super::providers::Entity)
                .from(Column::ProviderId)
                .to(super::providers::Column::Id)
                .into(),
            Self::AggIngredients => Entity::belongs_to(super::agg_ingredients::Entity)
                .from(Column::AggregateId)
                .to(super::agg_ingredients::Column::Id)
                .into(),
        }
    }
}

impl Related<super::providers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Providers.def()
    }
}

impl Related<super::agg_ingredients::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AggIngredients.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
