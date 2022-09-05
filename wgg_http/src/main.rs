use tracing_subscriber::util::SubscriberInitExt;
use wgg_http::setup::Application;
use wgg_http::{config, get_quit_notifier, telemetry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup Tracing
    let subscriber = telemetry::create_subscriber("DEBUG,wgg_http=TRACE,wgg_providers=TRACE,sqlx=WARN,hyper=WARN");
    subscriber.init();

    // Setup server
    let config = config::initialise_config()?;
    let app = Application::new(config).await?;

    let notifier = get_quit_notifier();

    app.run(notifier).await?;

    Ok(())
}
