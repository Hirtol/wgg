use crate::WggProvider;
use std::sync::Arc;
use wgg_scheduler::JobScheduler;

pub fn schedule_all_jobs(scheduler: &JobScheduler, providers: Arc<WggProvider>) {
    let sale_schedule = "0 0 0 * * *".try_into().unwrap();
    let sale_job = crate::caching::scheduled::get_promotions_refresh_job(sale_schedule, providers);

    let _ = scheduler.push(sale_job);
}
