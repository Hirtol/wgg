use crate::job::{Job, JobId};
use crate::{error, JobScheduler};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::time::Instant;

#[derive(Debug)]
pub enum Messages {
    AddJob(JobId, Job),
    RemoveJob(JobId),
    PauseScheduler,
    ResumeScheduler,
    PauseJob(JobId),
    ResumeJob(JobId),
}

pub struct RunnerState {
    pub jobs: HashMap<JobId, Job>,
    pub main_ref: JobScheduler,
    pub recv: tokio::sync::mpsc::UnboundedReceiver<Messages>,
    pub quit_notify: Arc<tokio::sync::Notify>,
    is_paused: bool,
    /// How often we should check
    check_rate: Duration,
}

impl RunnerState {
    pub fn new(
        jobs: HashMap<JobId, Job>,
        main_ref: JobScheduler,
        recv: UnboundedReceiver<Messages>,
        quitter: Arc<tokio::sync::Notify>,
    ) -> Self {
        Self {
            jobs,
            main_ref,
            recv,
            quit_notify: quitter,
            is_paused: false,
            check_rate: Duration::from_millis(500),
        }
    }

    pub async fn run(mut self) -> error::Result<()> {
        'mainloop: loop {
            tokio::select! {
                _ = self.quit_notify.notified() => {
                    break 'mainloop;
                } ,
                _ = tokio::time::sleep_until(Instant::now() + self.check_rate), if !self.is_paused => {
                    let now = Utc::now();
                    #[cfg(feature = "tracing")]
                    tracing::trace!(?now, "Scheduler checking jobs...");

                    for (id, job) in self.jobs.iter_mut() {
                        if job.is_pending(now) {
                            // Ignore errors for now
                            let _ = job.run(*id, self.main_ref.clone()).await;
                        }
                    }
                }
                Some(msg) = self.recv.recv() => {
                    self.handle_msg(msg).await?;
                }
            }
        }
        Ok(())
    }

    async fn handle_msg(&mut self, msg: Messages) -> anyhow::Result<()> {
        match msg {
            Messages::AddJob(id, job) => {
                self.jobs.insert(id, job);
            }
            Messages::RemoveJob(id) => {
                self.jobs.remove(&id);
            }
            Messages::PauseScheduler => self.is_paused = true,
            Messages::ResumeScheduler => self.is_paused = false,
            Messages::PauseJob(id) => {
                if let Some(job) = self.jobs.get_mut(&id) {
                    job.set_paused(true)
                }
            }
            Messages::ResumeJob(id) => {
                if let Some(job) = self.jobs.get_mut(&id) {
                    job.set_paused(false)
                }
            }
        }

        Ok(())
    }
}
