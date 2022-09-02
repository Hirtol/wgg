use crate::schema::{agg_ingredients, agg_ingredients_links, providers};
use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Selectable, Queryable, Debug, Identifiable)]
#[diesel(table_name = agg_ingredients)]
pub struct AggIngredient {
    pub id: i32,
    pub name: String,
    pub created_by: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Selectable, Queryable, Debug, Identifiable, Associations)]
#[diesel(belongs_to(AggIngredient, foreign_key = id))]
#[diesel(primary_key(id, provider_id))]
#[diesel(table_name = agg_ingredients_links)]
pub struct AggIngredientsLink {
    pub id: i32,
    pub provider_id: i32,
    pub provider_ingr_id: String,
}

#[derive(Selectable, Queryable, Debug)]
#[diesel(table_name = providers)]
pub struct Provider {
    pub id: i32,
    pub name: String,
}
