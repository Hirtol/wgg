use crate::Provider;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("Could not initialise providers: {0}")]
    InitialisationFailed(String),
    #[error("No responses had any content")]
    NothingFound,
    #[error("The requested provider wasn't initialised: {0:?}")]
    ProviderUninitialised(Provider),
    #[error(transparent)]
    PicnicError(#[from] wgg_picnic::ApiError),
    #[error(transparent)]
    JumboError(#[from] wgg_jumbo::ApiError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}
