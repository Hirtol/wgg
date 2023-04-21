//! Re-exports all scheduled jobs of sub-features.
use crate::api::{auth, cart, AppState};
use wgg_scheduler::JobScheduler;

/// Schedule all relevant jobs for this API.
pub fn schedule_all_jobs(scheduler: &JobScheduler, state: AppState) {
    let cart_data_schedule = "0 0 * * * * *".try_into().unwrap();
    let auth_token_schedule = "0 0 * * * * *".try_into().unwrap();

    let cart_job = cart::scheduled_jobs::create_job_keep_cart_data_fresh(cart_data_schedule, state.clone());
    let token_job = auth::scheduled_jobs::create_remove_expired_auth_tokens(auth_token_schedule, state);

    scheduler.push(cart_job);
    scheduler.push(token_job);
}
