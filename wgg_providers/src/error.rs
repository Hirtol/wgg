use crate::error::SubProviderError::{JumboMiscError, PicnicMiscError};
use crate::models::Provider;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, ProviderError>;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("Could not initialise providers: {0}")]
    InitialisationFailed(String),
    #[error("No responses had any content")]
    NothingFound,
    #[error("The requested provider wasn't initialised: {0:?}")]
    ProviderUninitialised(Provider),
    #[error("Provider: {0:?} - Failure: {1:?}")]
    SubProviderError(Provider, SubProviderError),
    #[error("Operation `{0}` is not supported on this provider")]
    OperationUnsupported(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}

impl From<wgg_picnic::ApiError> for ProviderError {
    fn from(value: wgg_picnic::ApiError) -> Self {
        Self::SubProviderError(Provider::Picnic, value.into())
    }
}
impl From<wgg_jumbo::ApiError> for ProviderError {
    fn from(value: wgg_jumbo::ApiError) -> Self {
        Self::SubProviderError(Provider::Jumbo, value.into())
    }
}

#[derive(Debug, Error)]
pub enum SubProviderError {
    #[error("Could not log in due to {0}")]
    LoginFailed(String),
    #[error("The requested resource (`{0}`) does not exist")]
    NotFound(String),
    #[error(transparent)]
    PicnicMiscError(wgg_picnic::ApiError),
    #[error(transparent)]
    JumboMiscError(wgg_jumbo::ApiError),
}

impl From<wgg_picnic::ApiError> for SubProviderError {
    fn from(value: wgg_picnic::ApiError) -> Self {
        match value {
            wgg_picnic::ApiError::LoginFailed(val) => SubProviderError::LoginFailed(val),
            wgg_picnic::ApiError::NotFound(url_suffix) => SubProviderError::NotFound(url_suffix),
            _ => PicnicMiscError(value),
        }
    }
}

impl From<wgg_jumbo::ApiError> for SubProviderError {
    fn from(value: wgg_jumbo::ApiError) -> Self {
        match value {
            wgg_jumbo::ApiError::LoginFailed(val) => SubProviderError::LoginFailed(val),
            wgg_jumbo::ApiError::NotFound(url_suffix) => SubProviderError::NotFound(url_suffix),
            _ => JumboMiscError(value),
        }
    }
}
