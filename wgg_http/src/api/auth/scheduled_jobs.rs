use sea_orm::{EntityTrait, QueryFilter};

use wgg_scheduler::schedule::Schedule;
use wgg_scheduler::Job;

use crate::api::AppState;
use crate::db;

pub fn create_remove_expired_auth_tokens(schedule: Schedule, state: AppState) -> Job {
    Job::new(schedule, move |_, _| {
        let state = state.clone();
        async move {
            let span = tracing::span!(tracing::Level::DEBUG, "Scheduled Job - Expire Auth Tokens");
            let _enter = span.enter();

            let delete = db::users_tokens::Entity::delete_many()
                .filter(db::users_tokens::non_expired().not())
                .exec(&state.db)
                .await?;

            tracing::debug!(expired = delete.rows_affected, "Expired auth tokens");

            Ok(())
        }
    })
    .unwrap()
}
