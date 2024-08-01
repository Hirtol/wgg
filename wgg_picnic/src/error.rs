use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Could not log in due to {0}")]
    LoginFailed(String),
    #[error("Picnic does not/no longer has the requested (`{0}`) resource")]
    NotFound(String),
    #[error("Can't request an empty search query")]
    EmptySearch,
    #[error("Either the current auth token is incorrect or has expired")]
    AuthError,
    #[error("The current credentials cache has no way of returning a second factor code")]
    NoSecondFactorCode,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}
