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
}
