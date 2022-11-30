use crate::error::ScheduleError;
use chrono::{DateTime, TimeZone};

#[derive(Debug, Clone)]
pub enum Schedule {
    /// A periodic execution
    Periodic(cron::Schedule),
    /// Set to execute at exactly this point in the future, once.
    Interval(chrono::Duration),
}

impl Schedule {
    /// Determine when the next execution of this schedule should take place.
    pub fn next<T: TimeZone>(&self, after: DateTime<T>) -> Option<DateTime<T>> {
        match self {
            Schedule::Periodic(schedule) => schedule.after(&after).next(),
            Schedule::Interval(inter) => Some(after + *inter),
        }
    }
}

impl TryFrom<&str> for Schedule {
    type Error = ScheduleError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self::Periodic(value.parse()?))
    }
}

impl TryFrom<std::time::Duration> for Schedule {
    type Error = ScheduleError;

    fn try_from(value: std::time::Duration) -> Result<Self, Self::Error> {
        let chrono_duration = chrono::Duration::from_std(value).map_err(|_| ScheduleError::OutOfRange)?;

        Ok(Self::Interval(chrono_duration))
    }
}

impl From<chrono::Duration> for Schedule {
    fn from(value: chrono::Duration) -> Self {
        Self::Interval(value)
    }
}
