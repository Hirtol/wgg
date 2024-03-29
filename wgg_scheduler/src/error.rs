use crate::JobId;
use std::convert::Infallible;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, ScheduleError>;

#[derive(Error, Debug)]
pub enum ScheduleError {
    #[error(transparent)]
    ParseError(#[from] cron::error::Error),
    #[error(transparent)]
    MiscError(#[from] anyhow::Error),
    #[error("Provided STD time was outside of a valid range")]
    OutOfRange,
    #[error("Failed to properly stop runner, it seems that it was already stopped?")]
    StopFailure,
    #[error("Failed to run job (`{0}`) because it was already busy")]
    JobAlreadyBusy(JobId),
}

impl From<Infallible> for ScheduleError {
    fn from(_: Infallible) -> Self {
        panic!("Infallible can't be constructed")
    }
}
