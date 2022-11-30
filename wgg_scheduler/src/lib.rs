pub mod error;
pub mod job;
mod runner;
pub mod schedule;
mod scheduler;

pub use scheduler::JobScheduler;

#[cfg(test)]
mod tests {
    use crate::job::Job;
    use crate::scheduler::JobScheduler;
    use chrono::Utc;
    use std::time::Duration;

    #[tokio::test]
    pub async fn basic_test() {
        let scheduler = JobScheduler::new();
        let job = Job::new("* 0/1 * * * *", |id, sched| {
            Box::pin(async move {
                println!("Hello World: {} - now: {:?}", id, Utc::now());
                let p = sched.remove(id);
                sched.add(
                    Job::new(Duration::from_secs(2), |_, _| {
                        Box::pin(async {
                            println!("HYYYYY");
                            Ok(())
                        })
                    })
                    .unwrap(),
                );
                println!("{:?}", p);
                Ok(())
            })
        })
        .unwrap();

        let job = scheduler.add(job);

        println!("JOB ADDED: {:?}", job);

        scheduler.start().await;

        tokio::time::sleep(Duration::from_secs(2)).await;
        // scheduler.stop();
        println!("DONE SLEEPING");
    }
}
