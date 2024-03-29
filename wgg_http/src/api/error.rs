use anyhow::anyhow;
use std::fmt::Debug;

use async_graphql::ErrorExtensions;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sea_orm::DbErr;
use thiserror::Error;
use tokio::task::JoinError;
use tracing::log;
use wgg_providers::ProviderError;

#[derive(Error, Debug)]
pub enum GraphqlError {
    /// Whenever a resource can't be found
    #[error("Could not find requested resource")]
    ResourceNotFound,
    /// An internal error which we don't want to elaborate too much on (additional details provided in String)
    #[error("An internal error occurred. Please try again later")]
    InternalError(String),
    /// User display error, provided `String` is displayed.
    #[error("Error: {0}")]
    UserError(String),
    #[error("Not allowed to perform this action")]
    Unauthorized,
    #[error("Invalid input provided by user")]
    InvalidInput(String),
    #[error("An internal error occurred. Please try again later")]
    Other(#[from] anyhow::Error),
    #[error("GraphQL Playground is disabled on this server")]
    PlaygroundDisabled,
}

impl GraphqlError {
    fn extensions(&self) -> async_graphql::ErrorExtensionValues {
        let mut e = async_graphql::ErrorExtensionValues::default();

        match self {
            GraphqlError::InternalError(reason) => e.set("details", reason.as_str()),
            GraphqlError::InvalidInput(input) => e.set("details", input.as_str()),
            GraphqlError::Other(default_err) => e.set("details", default_err.to_string()),
            _ => {}
        }

        e
    }

    fn status_code(&self) -> axum::http::StatusCode {
        match self {
            GraphqlError::ResourceNotFound => StatusCode::NOT_FOUND,
            GraphqlError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            GraphqlError::UserError(_) => StatusCode::BAD_REQUEST,
            GraphqlError::Unauthorized => StatusCode::UNAUTHORIZED,
            GraphqlError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            GraphqlError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
            GraphqlError::PlaygroundDisabled => StatusCode::NOT_FOUND,
        }
    }
}

impl From<GraphqlError> for async_graphql::Error {
    fn from(value: GraphqlError) -> Self {
        async_graphql::Error {
            message: value.to_string(),
            extensions: Some(value.extensions()),
            source: Some(std::sync::Arc::new(value)),
        }
    }
}

impl ErrorExtensions for GraphqlError {
    fn extend(&self) -> async_graphql::Error {
        let mut output = async_graphql::Error::new(self.to_string());
        output.extensions = Some(self.extensions());
        output
    }
}

impl IntoResponse for GraphqlError {
    fn into_response(self) -> Response {
        match self {
            GraphqlError::InternalError(_) => {
                log::warn!("Internal error: {}", self.to_string());
            }
            GraphqlError::Other(_) => {
                log::warn!("Misc Error: {:?}", self);
            }
            _ => {}
        }

        match self {
            GraphqlError::InvalidInput(ref errors) => {
                let response = ApiResponseError {
                    code: self.status_code().as_u16(),
                    message: self.to_string(),
                    details: Some(errors),
                };

                (self.status_code(), axum::Json(response)).into_response()
            }
            _ => {
                let response = ApiResponseError::<()> {
                    code: self.status_code().as_u16(),
                    message: self.to_string(),
                    details: None,
                };

                (self.status_code(), axum::Json(response)).into_response()
            }
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
struct ApiResponseError<T> {
    pub code: u16,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<T>,
}

impl Clone for GraphqlError {
    fn clone(&self) -> Self {
        match self {
            GraphqlError::ResourceNotFound => GraphqlError::ResourceNotFound,
            GraphqlError::InternalError(e) => GraphqlError::InternalError(e.clone()),
            GraphqlError::UserError(e) => GraphqlError::UserError(e.clone()),
            GraphqlError::Unauthorized => GraphqlError::Unauthorized,
            GraphqlError::InvalidInput(e) => GraphqlError::InvalidInput(e.clone()),
            GraphqlError::Other(e) => GraphqlError::Other(anyhow::Error::msg(e.to_string())),
            GraphqlError::PlaygroundDisabled => GraphqlError::PlaygroundDisabled,
        }
    }
}

impl From<sqlx::Error> for GraphqlError {
    fn from(e: sqlx::Error) -> Self {
        use sqlx::Error;
        match e {
            Error::RowNotFound => GraphqlError::ResourceNotFound,
            _ => GraphqlError::InternalError(e.to_string()),
        }
    }
}

impl From<std::io::Error> for GraphqlError {
    fn from(e: std::io::Error) -> Self {
        GraphqlError::InternalError(e.to_string())
    }
}

impl From<async_graphql::Error> for GraphqlError {
    fn from(e: async_graphql::Error) -> Self {
        Self::InternalError(e.message)
    }
}

impl From<String> for GraphqlError {
    fn from(e: String) -> Self {
        Self::UserError(e)
    }
}

impl From<&str> for GraphqlError {
    fn from(e: &str) -> Self {
        Self::UserError(e.to_string())
    }
}

impl From<JoinError> for GraphqlError {
    fn from(e: JoinError) -> Self {
        Self::InternalError(e.to_string())
    }
}

impl From<sea_orm::DbErr> for GraphqlError {
    fn from(e: sea_orm::DbErr) -> Self {
        match e {
            DbErr::RecordNotFound(_) => Self::ResourceNotFound,
            _ => Self::InternalError(e.to_string()),
        }
    }
}

impl<T: std::error::Error> From<sea_orm::TransactionError<T>> for GraphqlError {
    fn from(e: sea_orm::TransactionError<T>) -> Self {
        Self::InternalError(e.to_string())
    }
}

impl From<wgg_providers::ProviderError> for GraphqlError {
    fn from(e: ProviderError) -> Self {
        match e {
            ProviderError::InitialisationFailed(e) => GraphqlError::InternalError(e),
            ProviderError::NothingFound => GraphqlError::ResourceNotFound,
            ProviderError::ProviderUninitialised(provider) => {
                GraphqlError::InternalError(format!("Provider uninitialised: {provider:?}"))
            }
            ProviderError::Other(e) => GraphqlError::Other(e),
            ProviderError::Reqwest(e) => GraphqlError::Other(anyhow!(e)),
            ProviderError::SubProviderError(provider, e) => {
                GraphqlError::Other(anyhow!("{:?} - Failure: {:#}", provider, e))
            }
            ProviderError::OperationUnsupported(msg) => GraphqlError::UserError(msg),
        }
    }
}
