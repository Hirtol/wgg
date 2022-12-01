pub mod error;
pub mod job;
mod runner;
pub mod schedule;
mod scheduler;

pub use scheduler::JobScheduler;

#[cfg(test)]
mod tests {
    use crate::error::ScheduleError;
    use crate::job::Job;
    use crate::schedule::Schedule;
    use crate::scheduler::JobScheduler;
    use chrono::Utc;
    use std::time::Duration;
    use tokio::sync::mpsc::UnboundedReceiver;

    #[tokio::test]
    pub async fn test_basic() {
        let scheduler = JobScheduler::new(basic_checking()).await;
        let (job, mut recv) = get_basic_job(basic_schedule());
        let _ = scheduler.add(job);
        scheduler.start().await;

        // Sleep enough time for the scheduler to be called
        basic_sleep().await;

        assert!(recv.recv().await.is_some());
    }

    #[tokio::test]
    pub async fn test_stop() {
        let scheduler = JobScheduler::new(basic_checking()).await;
        let (job, mut recv) = get_basic_job(basic_schedule());
        let _ = scheduler.add(job);
        scheduler.start().await;

        let _ = scheduler.stop().await;

        // Sleep enough time for the scheduler to be called
        basic_sleep().await;

        assert!(recv.try_recv().is_err());
    }

    #[tokio::test]
    pub async fn test_pause() {
        let scheduler = JobScheduler::new(basic_checking()).await;
        let (job, mut recv) = get_basic_job(basic_schedule());
        let _ = scheduler.add(job);
        scheduler.start().await;

        scheduler.pause();

        // Sleep enough time for the scheduler to be called
        basic_sleep().await;

        assert!(recv.try_recv().is_err());

        scheduler.resume();

        basic_sleep().await;

        assert!(recv.try_recv().is_ok());
    }

    #[tokio::test]
    pub async fn test_pause_job() {
        let scheduler = JobScheduler::new(basic_checking()).await;
        let (job, mut recv) = get_basic_job(basic_schedule());
        let (job2, mut recv2) = get_basic_job(basic_schedule());
        let id = scheduler.add(job);
        let _ = scheduler.add(job2);
        scheduler.start().await;

        scheduler.pause_job(id);

        // Sleep enough time for the scheduler to be called
        basic_sleep().await;

        assert!(recv.try_recv().is_err());
        assert!(recv2.try_recv().is_ok());

        scheduler.resume_job(id);

        basic_sleep().await;

        assert!(recv.try_recv().is_ok());
        assert!(recv2.try_recv().is_ok());
    }

    fn get_basic_job(schedule: impl TryInto<Schedule, Error = ScheduleError>) -> (Job, UnboundedReceiver<()>) {
        let (send, recv) = tokio::sync::mpsc::unbounded_channel();
        let job = Job::new(schedule, move |id, _| {
            let snd = send.clone();
            Box::pin(async move {
                println!("Basic Job: {} - now: {:?}", id, Utc::now());
                snd.send(()).unwrap();
                Ok(())
            })
        })
        .unwrap();

        (job, recv)
    }

    fn basic_checking() -> Duration {
        Duration::from_millis(1)
    }

    fn basic_schedule() -> impl TryInto<Schedule, Error = ScheduleError> {
        basic_checking()
    }

    async fn basic_sleep() {
        // Enough time to at least be woken up by the OS once
        tokio::time::sleep(basic_checking() * 50).await
    }
}
