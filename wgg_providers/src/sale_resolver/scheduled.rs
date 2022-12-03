use crate::sale_resolver::refresh_promotions;
use crate::WggProvider;
use futures::FutureExt;
use std::sync::Arc;
use wgg_scheduler::schedule::Schedule;
use wgg_scheduler::Job;

pub fn get_promotions_refresh_job(schedule: Schedule, providers: Arc<WggProvider>) -> Job {
    Job::new(schedule, move |_, _| {
        let providers = providers.clone();
        async move {
            let span = tracing::span!(tracing::Level::DEBUG, "Scheduled Job - Promotion Data");
            let _enter = span.enter();

            for provider in providers.active_providers() {
                let real_provider = provider.provider();
                tracing::debug!(provider=?real_provider, "Refreshing promotion data for provider");
                refresh_promotions(&providers, real_provider).await?
            }
            Ok(())
        }
        .boxed()
    })
    .expect("Invalid schedule provided")
}
