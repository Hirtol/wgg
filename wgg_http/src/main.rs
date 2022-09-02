use wgg_http::setup::Application;
use wgg_http::{config, get_quit_notifier};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::initialise_config()?;
    let app = Application::new(config).await?;

    let notifier = get_quit_notifier();

    app.run(notifier).await?;

    Ok(())
}
