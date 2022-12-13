use crate::error::Result;
use crate::error::*;
use crate::schedule::Schedule;
use crate::scheduler::JobScheduler;
use chrono::{DateTime, Utc};
use futures::future::BoxFuture;
use futures::FutureExt;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use uuid::Uuid;

pub type JobId = Uuid;

pub struct Job {
    pub(crate) id: JobId,
    pub(crate) function: Box<dyn Send + Sync + FnMut(JobId, JobScheduler) -> BoxFuture<'static, anyhow::Result<()>>>,
    pub(crate) schedule: Schedule,
    pub(crate) next_run_at: DateTime<Utc>,
}

impl Job {
    /// Create a new [Job], the provided schedule can either be a CRON string (with seconds!), or a [Duration](std::time::Duration)
    /// from either `chrono` or `std` if a single delayed execution is desired.
    ///
    /// # Returns
    ///
    /// The created job so long as the schedule (CRON string) is valid.
    pub fn new<Fn, T, Fut>(schedule: T, mut func: Fn) -> Result<Job>
    where
        T: TryInto<Schedule>,
        Fn: Clone + Send + Sync + FnMut(JobId, JobScheduler) -> Fut + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
        ScheduleError: From<T::Error>,
    {
        let schedule = schedule.try_into()?;
        Ok(Job {
            id: Uuid::new_v4(),
            function: Box::new(move |job_id, job_sched| func(job_id, job_sched).boxed()),
            next_run_at: schedule.next(Utc::now()).ok_or(ScheduleError::OutOfRange)?,
            schedule,
        })
    }

    pub fn next_run(&self) -> DateTime<Utc> {
        self.next_run_at
    }

    /// Check whether this job is currently pending execution
    pub fn is_pending(&self, now: DateTime<Utc>) -> bool {
        now > self.next_run_at
    }

    /// Run the current job and update the next time to execute
    pub async fn run(&mut self, job_id: JobId, scheduler: JobScheduler) -> Result<()> {
        let result = (self.function)(job_id, scheduler).await;

        self.next_run_at = self.schedule.next(Utc::now()).ok_or(ScheduleError::OutOfRange)?;

        Ok(result?)
    }
}

impl Debug for Job {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        core::fmt::Formatter::debug_struct(f, "Job")
            .field("id", &self.id)
            .field("next_run_at", &self.next_run_at)
            .field("schedule", &self.schedule)
            .finish()
    }
}
