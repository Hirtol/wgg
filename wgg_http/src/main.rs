use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use wgg_http::setup::Application;
use wgg_http::{config, get_quit_notifier};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup Tracing
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("DEBUG,wgg_http=TRACE,sqlx=WARN,hyper=WARN"));
    let format = tracing_subscriber::fmt::format()
        .with_level(true)
        .with_thread_ids(true)
        .with_source_location(true)
        .with_ansi(true);

    let format_layer = tracing_subscriber::fmt::layer().event_format(format);

    tracing_subscriber::registry().with(filter).with(format_layer).init();

    // Setup server
    let config = config::initialise_config()?;
    let app = Application::new(config).await?;

    let notifier = get_quit_notifier();

    app.run(notifier).await?;

    Ok(())
}
