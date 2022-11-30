use crate::job::{Job, JobId};
use crate::{error, JobScheduler};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;

#[derive(Debug)]
pub enum Messages {
    AddJob(JobId, Job),
    RemoveJob(JobId),
}

pub struct RunnerState {
    pub jobs: HashMap<JobId, Job>,
    pub main_ref: JobScheduler,
    pub recv: tokio::sync::mpsc::UnboundedReceiver<Messages>,
    pub quit_notify: Arc<tokio::sync::Notify>,
}

impl RunnerState {
    pub async fn run(mut self) -> error::Result<()> {
        'mainloop: loop {
            tokio::select! {
                _ = self.quit_notify.notified() => {
                    break 'mainloop;
                } ,
                _ = tokio::time::sleep_until(Instant::now() + Duration::from_millis(500)) => {
                    tracing::trace!("Scheduler checking jobs...");

                    for (id, job) in self.jobs.iter_mut() {
                        if job.is_pending() {
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
        }

        Ok(())
    }
}
