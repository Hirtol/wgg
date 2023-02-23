//! Re-exports all scheduled jobs of sub-features.
pub use super::cart::scheduled_jobs;
use crate::api::AppState;
use wgg_scheduler::JobScheduler;

/// Schedule all relevant jobs for this API.
pub fn schedule_all_jobs(scheduler: &JobScheduler, state: AppState) {
    let cart_data_schedule = "0 0 * * * * *".try_into().unwrap();
    let job = scheduled_jobs::create_job_keep_cart_data_fresh(cart_data_schedule, state);
    scheduler.push(job);
}
