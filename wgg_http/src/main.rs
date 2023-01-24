use tracing_subscriber::util::SubscriberInitExt;
use wgg_http::setup::Application;
use wgg_http::{config, get_quit_notifier, telemetry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // We don't care if it can't find a .env file
    let _ = dotenv::dotenv();

    // Setup Tracing
    let subscriber = telemetry::create_subscriber("DEBUG,wgg_http=TRACE,wgg_providers=TRACE,sqlx=WARN,hyper=WARN");
    subscriber.init();

    // Setup server
    let config = config::initialise_config()?;
    let app = Application::new(config).await?;

    let notifier = get_quit_notifier();

    let final_config = app.run(notifier).await?;

    // Save config when we're done
    let _ = config::save_config(&final_config);

    Ok(())
}
