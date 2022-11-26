use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Could not log in due to {0}")]
    LoginFailed(String),
    #[error("The requested resource did not exist: {0}")]
    NotFound(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}
