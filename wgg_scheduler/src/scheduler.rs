use crate::error::Result;
use crate::job::{Job, JobId};
use crate::runner::{Messages, RunnerState};
use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct JobScheduler {
    inner: Arc<SchedulerInner>,
}

struct SchedulerInner {
    pub running: Mutex<Option<RunningDetails>>,
    pub job_backlog: Mutex<HashMap<JobId, Job>>,
}

struct RunningDetails {
    pub handle: tokio::task::JoinHandle<Result<()>>,
    pub quit_notify: Arc<tokio::sync::Notify>,
    pub snd: tokio::sync::mpsc::UnboundedSender<Messages>,
}

impl JobScheduler {
    pub fn new() -> JobScheduler {
        JobScheduler {
            inner: Arc::new(SchedulerInner::default()),
        }
    }

    /// Add a new [Job] to the scheduler.
    ///
    /// This can be called whenever, even if the scheduler is already running
    ///
    /// # Example
    ///
    /// ```norun
    /// # use wgg_scheduler::*;
    /// # use wgg_scheduler::job::Job;
    /// let scheduler = JobScheduler::new();
    /// let job = Job::new("* 0/5 * * * * *", |job_id, scheduler| Box::pin(async move {
    ///     println!("Hello World Every 5 Minutes!");
    ///     // Stop self after the first execution
    ///     scheduler.remove(job_id);
    ///     Ok(())
    /// }))?;
    ///
    /// let job_id = scheduler.add(job);
    /// // Start executing the above job
    /// scheduler.start();
    /// ```
    pub fn add(&self, job: Job) -> JobId {
        let uuid = Uuid::new_v4();

        let lock = self.inner.running.lock().unwrap();

        if let Some(runner) = lock.as_ref() {
            // The unwrap will never be hit as we'd have no runner!
            runner.snd.send(Messages::AddJob(uuid, job)).ok().unwrap();
        } else {
            // Queue up jobs for when we start the scheduler.
            let mut queue = self.inner.job_backlog.lock().unwrap();
            queue.insert(uuid, job);
        }

        uuid
    }

    /// Remove a job with the given `job_id`.
    pub fn remove(&self, job_id: JobId) -> Option<()> {
        let lock = self.inner.running.lock().unwrap();

        if let Some(runner) = lock.as_ref() {
            runner.snd.send(Messages::RemoveJob(job_id)).ok()
        } else {
            // Queue up jobs for when we start the scheduler.
            let mut queue = self.inner.job_backlog.lock().unwrap();
            queue.remove(&job_id).map(|_| ())
        }
    }

    /// Start the JobScheduler, and start executing on a new runner
    ///
    /// # Runner
    ///
    /// This will launch a separate [task](tokio::task::JoinHandle) which will poll every `500ms` for a new task to execute.
    ///
    /// # Returns
    ///
    /// Should this `start` function be called again whilst this existing runner exists it will return immediately with `false`.
    /// `true` if a new runner has been successfully started.
    pub async fn start(&self) -> bool {
        let notify = Arc::new(tokio::sync::Notify::new());
        let (snd, recv) = tokio::sync::mpsc::unbounded_channel();

        let backlog = std::mem::take(&mut *self.inner.job_backlog.lock().unwrap());
        let runner = RunnerState::new(backlog, (*self).clone(), recv, notify.clone());
        // Since we're spawning a future here best ensure we're running in a run-time!
        // This is why the function is marked as `async`.
        let handle = tokio::task::spawn(runner.run());

        let mut inner_lock = self.inner.running.lock().unwrap();

        // First ensure we've stopped the current runner if it's present
        if inner_lock.is_some() {
            return false;
        }

        *inner_lock = Some(RunningDetails {
            handle,
            quit_notify: notify,
            snd,
        });

        true
    }

    /// Will force the current scheduler to a stop.
    ///
    /// Is automatically called in the [JobScheduler]'s `drop` implementation.
    /// Can be called several times without consequence.
    ///
    /// All current jobs are dropped when this is called. If one wants to re-start the scheduler later on one will need
    /// to re-add them all.
    ///
    /// # Returns
    ///
    /// An (optional) future which one can await in order to ensure the scheduler has completely shut down.
    /// If the scheduler has already been stopped this will return `None` instead.
    pub fn stop(&self) -> Option<impl Future<Output = std::result::Result<Result<()>, tokio::task::JoinError>>> {
        // As we want to have this stop on the inner data's drop we'll have to defer implementation to that type.
        self.inner.stop()
    }

    /// Pause the entire scheduler, needs to be manually [unpaused](Self::unpause).
    pub fn pause(&self) {
        let lock = self.inner.running.lock().unwrap();

        if let Some(runner) = lock.as_ref() {
            let _ = runner.snd.send(Messages::PauseScheduler).ok();
        }
    }

    /// Resume the scheduler after it was [paused](Self::pause).
    pub fn resume(&self) {
        let lock = self.inner.running.lock().unwrap();

        if let Some(runner) = lock.as_ref() {
            let _ = runner.snd.send(Messages::ResumeScheduler).ok();
        }
    }

    /// Pause a single job on the scheduler, needs to be manually [unpaused](Self::resume_job).
    pub fn pause_job(&self, job_id: JobId) {
        let lock = self.inner.running.lock().unwrap();

        if let Some(runner) = lock.as_ref() {
            let _ = runner.snd.send(Messages::PauseJob(job_id)).ok();
        }
    }

    /// Resume the given job if it had been paused.
    pub fn resume_job(&self, job_id: JobId) {
        let lock = self.inner.running.lock().unwrap();

        if let Some(runner) = lock.as_ref() {
            let _ = runner.snd.send(Messages::ResumeJob(job_id)).ok();
        }
    }
}

impl SchedulerInner {
    fn stop(&self) -> Option<impl Future<Output = std::result::Result<Result<()>, tokio::task::JoinError>>> {
        let details = self.running.lock().unwrap().take();

        if let Some(details) = details {
            // Signal the actor to stop, and subsequently wait for it to do so.
            details.quit_notify.notify_one();

            Some(details.handle)
        } else {
            None
        }
    }
}

impl Drop for SchedulerInner {
    fn drop(&mut self) {
        // Forcefully ensure that we've stopped the runner.
        self.stop();
    }
}

impl Default for SchedulerInner {
    fn default() -> Self {
        SchedulerInner {
            running: Mutex::new(None),
            job_backlog: Mutex::new(Default::default()),
        }
    }
}
