use std::fmt::Debug;

use async_graphql::ErrorExtensions;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;
use tokio::task::JoinError;
use tracing::log;

#[derive(Error, Debug)]
pub enum GraphqlError {
    /// Whenever a resource can't be found
    #[error("Could not find requested resource")]
    ResourceNotFound,
    /// An internal error which we don't want to elaborate too much on (additional details provided in String)
    #[error("An internal error occurred. Please try again later.")]
    InternalError(String),
    /// User display error, provided `String` is displayed.
    #[error("Error: {0}")]
    UserError(String),
    #[error("Not allowed to perform this action")]
    Unauthorized,
    #[error("Invalid Input Error: {0}")]
    InvalidInput(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl ErrorExtensions for GraphqlError {
    fn extend(&self) -> async_graphql::Error {
        async_graphql::Error::new(format!("{:#}", self)).extend_with(|_, e| match self {
            GraphqlError::InternalError(reason) => e.set("details", reason.as_str()),
            GraphqlError::Other(default_err) => e.set("details", default_err.to_string()),
            _ => {}
        })
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

impl GraphqlError {
    fn status_code(&self) -> axum::http::StatusCode {
        match self {
            GraphqlError::ResourceNotFound => StatusCode::NOT_FOUND,
            GraphqlError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            GraphqlError::UserError(_) => StatusCode::BAD_REQUEST,
            GraphqlError::Unauthorized => StatusCode::UNAUTHORIZED,
            GraphqlError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            GraphqlError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
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

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct ApiResponseError<T> {
    pub code: u16,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<T>,
}

impl<T> ApiResponseError<T> {
    pub fn new(code: u16, message: String) -> Self {
        ApiResponseError {
            code,
            message,
            details: None,
        }
    }
}