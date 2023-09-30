//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.3

use sea_orm::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "agg_ingredients"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Eq)]
pub struct Model {
    pub id: i32,
    pub name: String,
    pub image_url: Option<String>,
    pub created_by: i32,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    Name,
    ImageUrl,
    CreatedBy,
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
    AggIngredientsLinks,
    CartContentsAggregate,
    Users,
}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
            Self::Id => ColumnType::Integer.def(),
            Self::Name => ColumnType::String(None).def(),
            Self::ImageUrl => ColumnType::String(None).def().null(),
            Self::CreatedBy => ColumnType::Integer.def(),
            Self::CreatedAt => ColumnType::Timestamp.def(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::AggIngredientsLinks => Entity::has_many(super::agg_ingredients_links::Entity).into(),
            Self::CartContentsAggregate => Entity::has_many(super::cart_contents_aggregate::Entity).into(),
            Self::Users => Entity::belongs_to(super::users::Entity)
                .from(Column::CreatedBy)
                .to(super::users::Column::Id)
                .into(),
        }
    }
}

impl Related<super::agg_ingredients_links::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AggIngredientsLinks.def()
    }
}

impl Related<super::cart_contents_aggregate::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CartContentsAggregate.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
