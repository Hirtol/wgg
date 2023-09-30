//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.3

use sea_orm::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "cart_contents_provider"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Eq)]
pub struct Model {
    pub id: i32,
    pub cart_id: i32,
    pub provider_id: i32,
    pub provider_product: String,
    pub quantity: i32,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    CartId,
    ProviderId,
    ProviderProduct,
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
    Cart,
    Providers,
}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
            Self::Id => ColumnType::Integer.def(),
            Self::CartId => ColumnType::Integer.def(),
            Self::ProviderId => ColumnType::Integer.def(),
            Self::ProviderProduct => ColumnType::String(None).def(),
            Self::Quantity => ColumnType::Integer.def(),
            Self::CreatedAt => ColumnType::Timestamp.def(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Cart => Entity::belongs_to(super::cart::Entity)
                .from(Column::CartId)
                .to(super::cart::Column::Id)
                .into(),
            Self::Providers => Entity::belongs_to(super::providers::Entity)
                .from(Column::ProviderId)
                .to(super::providers::Column::Id)
                .into(),
        }
    }
}

impl Related<super::cart::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Cart.def()
    }
}

impl Related<super::providers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Providers.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
