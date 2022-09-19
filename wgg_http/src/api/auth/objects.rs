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