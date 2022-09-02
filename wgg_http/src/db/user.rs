use crate::schema::{users, users_tokens};
use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Identifiable, Selectable, Queryable, Debug)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub username: String,
    pub hash: String,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub email: String,
    pub username: String,
    pub hash: String,
}

#[derive(Selectable, Queryable, Debug, Associations)]
#[diesel(belongs_to(User))]
#[diesel(table_name = users_tokens)]
pub struct UsersToken {
    pub id: i32,
    pub user_id: i32,
    pub token: String,
    pub created: NaiveDateTime,
    pub expires: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = users_tokens)]
pub struct NewUserToken {
    pub user_id: i32,
    pub token: String,
    pub created: NaiveDateTime,
    pub expires: NaiveDateTime,
}
