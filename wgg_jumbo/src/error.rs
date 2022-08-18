use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Could not log in due to {0}")]
    LoginFailed(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}
