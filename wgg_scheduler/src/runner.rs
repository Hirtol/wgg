use crate::error::ScheduleError;
use crate::job::{Job, JobId};
use crate::{error, JobScheduler};
use chrono::{DateTime, Utc};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use keyed_priority_queue::KeyedPriorityQueue;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::JoinHandle;
use tokio::time::Instant;

type RunnerResult = Result<Job, (Job, ScheduleError)>;

#[derive(Debug)]
pub enum Messages {
    AddJob(JobId, Job),
    RemoveJob(JobId, tokio::sync::oneshot::Sender<Option<Job>>),
    PauseScheduler,
    ResumeScheduler,
    PauseJob(JobId),
    ResumeJob(JobId),
}

pub struct RunnerState {
    jobs: HashMap<JobId, JobWrapper>,
    job_queue: KeyedPriorityQueue<JobId, Reverse<DateTime<Utc>>>,
    main_ref: JobScheduler,
    recv: UnboundedReceiver<Messages>,
    quit_notify: Arc<tokio::sync::Notify>,
    is_paused: bool,
    /// How often we should check
    check_rate: Duration,
}

impl RunnerState {
    pub fn new(
        main_ref: JobScheduler,
        recv: UnboundedReceiver<Messages>,
        quitter: Arc<tokio::sync::Notify>,
        check_rate: Duration,
        is_paused: bool,
    ) -> Self {
        Self {
            jobs: Default::default(),
            job_queue: Default::default(),
            main_ref,
            recv,
            quit_notify: quitter,
            is_paused,
            check_rate,
        }
    }

    pub async fn run(mut self) -> error::Result<()> {
        let mut pipelines: FuturesUnordered<JoinHandle<RunnerResult>> = FuturesUnordered::new();

        'mainloop: loop {
            tokio::select! {
                _ = self.quit_notify.notified() => {
                    break 'mainloop;
                } ,
                _ = tokio::time::sleep_until(Instant::now() + self.check_rate), if !self.is_paused => {
                    self.check_pending_jobs(&pipelines)?;
                }
                Some(msg) = self.recv.recv() => {
                    self.handle_msg(msg).await?;
                }
                Some(joined) = pipelines.next() => {
                    if let Ok(joined_result) = joined {
                        self.rejoin_job(joined_result)
                    } else {
                        #[cfg(feature = "tracing")]
                        tracing::error!("Job was cancelled before being joined!");
                    }
                }
                else => break,
            }
        }
        Ok(())
    }

    async fn handle_msg(&mut self, msg: Messages) -> anyhow::Result<()> {
        match msg {
            Messages::AddJob(id, job) => {
                self.job_queue.push(id, Reverse(job.next_run()));
                self.jobs.insert(id, JobWrapper::new(job));
                println!("{:#?}", self.job_queue);
            }
            Messages::RemoveJob(id, response) => {
                self.job_queue.remove(&id);
                let job = self.jobs.remove(&id).and_then(JobWrapper::into_job_or_none);
                let _ = response.send(job);
            }
            Messages::PauseScheduler => self.is_paused = true,
            Messages::ResumeScheduler => self.is_paused = false,
            Messages::PauseJob(id) => {
                if let Some(job) = self.jobs.get_mut(&id) {
                    let _ = self.job_queue.remove(&id);
                    job.set_paused(true);
                }
            }
            Messages::ResumeJob(id) => {
                if let Some(job) = self.jobs.get_mut(&id) {
                    if let JobWrapper::Available { job, .. } = job {
                        self.job_queue.push(job.id, Reverse(job.next_run()));
                    }

                    job.set_paused(false);
                }
            }
        }

        Ok(())
    }

    fn check_pending_jobs(&mut self, pipelines: &FuturesUnordered<JoinHandle<RunnerResult>>) -> error::Result<()> {
        let now = Utc::now();
        #[cfg(feature = "tracing")]
        tracing::trace!(?now, "Scheduler checking jobs...");

        if matches!(self.job_queue.peek(), Some((_, when)) if now >= when.0) {
            let (id, _) = self.job_queue.pop().unwrap();
            let mut taken_job = self
                .jobs
                .get_mut(&id)
                .and_then(JobWrapper::make_busy)
                .ok_or_else(|| ScheduleError::JobAlreadyBusy(id))?;

            let main_ref = self.main_ref.clone();
            let future = async move {
                #[cfg(feature = "tracing")]
                tracing::trace!(?id, "Starting job: {:?}", id);

                if let Err(e) = taken_job.run(id, main_ref).await {
                    Err((taken_job, e))
                } else {
                    #[cfg(feature = "tracing")]
                    tracing::trace!(?id, "Done running job: {:?}", id);

                    Ok(taken_job)
                }
            };

            pipelines.push(tokio::task::spawn(future));
        }

        Ok(())
    }

    fn rejoin_job(&mut self, runner_result: RunnerResult) {
        let job = match runner_result {
            Ok(job) => job,
            Err((job, error)) => {
                #[cfg(feature = "tracing")]
                tracing::warn!(?error, "Job returned error");
                job
            }
        };

        // If the job was removed in the meantime we don't want to re-add it, hence the `get_mut`!
        let Some(wrapper) = self.jobs.get_mut(&job.id) else {
            return
        };

        if !wrapper.is_paused() {
            self.job_queue.push(job.id, Reverse(job.next_run()));
        }
        wrapper.make_available(job);
    }
}

#[derive(Debug)]
enum JobWrapper {
    Available { job: Job, paused: bool },
    Busy { paused: bool },
}

impl JobWrapper {
    pub fn new(job: Job) -> Self {
        Self::Available { paused: false, job }
    }

    pub fn set_paused(&mut self, set_paused: bool) {
        match self {
            JobWrapper::Available { paused, .. } => *paused = set_paused,
            JobWrapper::Busy { paused } => *paused = set_paused,
        }
    }

    #[allow(dead_code)]
    pub fn is_paused(&self) -> bool {
        match self {
            JobWrapper::Available { paused, .. } => *paused,
            JobWrapper::Busy { paused } => *paused,
        }
    }

    /// Check whether this job is pending by comparing it to `now`.
    ///
    /// Will automatically check if this job is paused or not.
    pub fn is_pending(&self, now: DateTime<Utc>) -> bool {
        match self {
            JobWrapper::Available { job, paused } => !*paused && job.is_pending(now),
            JobWrapper::Busy { .. } => false,
        }
    }

    /// Make this [JobWrapper::Available], even if it was already.
    ///
    /// Preserves paused state.
    pub fn make_available(&mut self, job: Job) {
        let paused = match self {
            JobWrapper::Available { paused, .. } => *paused,
            JobWrapper::Busy { paused } => *paused,
        };

        *self = JobWrapper::Available { job, paused }
    }

    /// Make `self` [JobWrapper::Busy], and return the [Job] if `self` was [JobWrapper::Available].
    ///
    /// Preserves paused state.
    pub fn make_busy(&mut self) -> Option<Job> {
        let current_job = std::mem::replace(self, JobWrapper::Busy { paused: false });

        let (result, paused) = match current_job {
            JobWrapper::Available { job, paused } => (Some(job), paused),
            JobWrapper::Busy { paused } => (None, paused),
        };

        *self = JobWrapper::Busy { paused };

        result
    }

    pub fn into_job_or_none(self) -> Option<Job> {
        match self {
            JobWrapper::Available { job, .. } => Some(job),
            JobWrapper::Busy { .. } => None,
        }
    }
}
