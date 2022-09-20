use crate::db;
use crate::db::Id;

/// Represents a user that is already logged in.
/// Implements [axum::extract::FromRequest] and can therefore be requested in HTTP service methods.
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, async_graphql::SimpleObject)]
pub struct AuthContext {
    pub id: Id,
    pub email: String,
    pub username: String,
    pub is_admin: bool,
}

impl From<db::users::Model> for AuthContext {
    fn from(model: db::users::Model) -> Self {
        AuthContext {
            id: model.id,
            email: model.email,
            username: model.username,
            is_admin: model.is_admin,
        }
    }
}