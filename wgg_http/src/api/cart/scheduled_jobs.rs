use crate::api::{ProductId, State};
use crate::db;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use std::collections::VecDeque;
use wgg_providers::models::Provider;
use wgg_scheduler::schedule::Schedule;
use wgg_scheduler::Job;

pub fn create_job_keep_cart_data_fresh(schedule: Schedule, state: State) -> Job {
    use futures::future::FutureExt;
    use futures::stream::StreamExt;
    Job::new(schedule, move |_, _| {
        let state = state.clone();
        async move {
            let span = tracing::span!(tracing::Level::DEBUG, "Scheduled Job - Cart Data");
            let _enter = span.enter();

            let mut job_queue = VecDeque::new();
            get_all_cart_products(&mut job_queue, &state.db, &state).await?;

            tracing::debug!(count = job_queue.len(), "Refreshing cache for items");
            let mut stream = futures::stream::iter(job_queue)
                .map(|(provider, product)| state.providers.search_product(provider, product))
                .buffer_unordered(2);

            while (stream.next().await).is_some() {}

            Ok(())
        }
        .boxed()
    })
    .unwrap()
}

async fn get_all_cart_products(
    queue: &mut VecDeque<(Provider, ProductId)>,
    db: &impl ConnectionTrait,
    state: &State,
) -> anyhow::Result<()> {
    let products = db::cart_contents::raw_product::Entity::find()
        .left_join(db::cart::Entity)
        .filter(db::cart::Column::CompletedAt.is_null())
        .all(db);

    let aggregate = db::cart_contents::aggregate::Entity::find()
        .left_join(db::cart::Entity)
        .filter(db::cart::Column::CompletedAt.is_null())
        .find_with_related(db::agg_ingredients_links::Entity)
        .all(db);

    let (products, aggregate) = futures::future::try_join(products, aggregate).await?;

    for product in products {
        let provider = state.provider_from_id(product.provider_id);

        queue.push_back((provider, product.provider_product));
    }

    for (_, products) in aggregate {
        for product in products {
            let provider = state.provider_from_id(product.provider_id);

            queue.push_back((provider, product.provider_ingr_id));
        }
    }

    Ok(())
}
