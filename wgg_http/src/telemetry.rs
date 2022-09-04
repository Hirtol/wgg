use tracing::Subscriber;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Layer};

/// Create the initial subscriber, alongside the custom formatting for standard i/o.
pub fn create_subscriber(default_directives: &str) -> impl Subscriber + Send + Sync {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_directives));
    let wgg_filter = tracing_subscriber::filter::filter_fn(|f| f.target().contains("wgg_"));

    let wgg_format = tracing_subscriber::fmt::format()
        .with_level(true)
        .with_thread_ids(true)
        .with_source_location(true)
        .with_ansi(true);
    let normal_format = wgg_format.clone().with_source_location(false);

    // We only want file locations in wgg_* logs, we therefore filter those out in the normal_logger.
    let wgg_logger = tracing_subscriber::fmt::layer()
        .event_format(wgg_format)
        .with_filter(wgg_filter)
        .with_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_directives)));
    let normal_logger = tracing_subscriber::fmt::layer()
        .event_format(normal_format)
        .with_filter(tracing_subscriber::filter::filter_fn(|m| !m.target().contains("wgg_")))
        .with_filter(env_filter);

    tracing_subscriber::registry().with(wgg_logger).with(normal_logger)
}
