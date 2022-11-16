use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Could not log in due to {0}")]
    LoginFailed(String),
    #[error("The requested resource did not exist")]
    NotFound,
    #[error("Can't request an empty search query")]
    EmptySearch,
    #[error("Either the current auth token is incorrect or has expired")]
    AuthError,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}
