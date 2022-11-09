use crate::api::dataloader::DataLoaders;
use crate::api::State;
use crate::config::{Config, DbConfig, SharedConfig};
use crate::db::Id;
use anyhow::Context;
use arc_swap::access::{DynAccess, DynGuard};
use arc_swap::ArcSwap;
use async_graphql::{EmptySubscription, Schema};
use axum::body::Body;
use axum::routing::{get_service, MethodRouter};
use sea_orm::{DatabaseConnection, SqlxSqliteConnector};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::SqlitePool;
use std::collections::BTreeMap;
use std::net::TcpListener;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::add_extension::AddExtensionLayer;
use tower_http::compression::CompressionLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use wgg_providers::models::Provider;
use wgg_providers::WggProvider;

mod caching;

pub struct Application {
    pub tcp: TcpListener,
    pub config: SharedConfig,
    pub db: DatabaseConnection,
    pub providers: Arc<WggProvider>,
    pub db_providers: BTreeMap<Provider, Id>,
}

impl Application {
    #[tracing::instrument(name = "Create application", skip(config), fields(addr = config.app.host, port = config.app.port))]
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let tcp = TcpListener::bind(config.app.bind_address())?;
        let db = initialise_database(&config.db).await?;

        setup_db_schema(&db).await?;

        let sea_db = SqlxSqliteConnector::from_sqlx_sqlite_pool(db);
        crate::utils::first_time_setup(&sea_db).await?;

        tracing::debug!("Creating Providers...");
        let cache = caching::setup_cache(&config).await;
        let mut providers = WggProvider::new(cache);

        if let Some(auth_token) = config.auth.picnic_auth_token.clone() {
            providers = providers.with_picnic(wgg_providers::PicnicCredentials::new(auth_token, "1".to_string()))
        } else if let Some((username, password)) = config
            .auth
            .picnic_username
            .clone()
            .zip(config.auth.picnic_password.clone())
        {
            providers = providers.with_picnic_login(&username, &password).await?;

            tracing::info!(auth_token=?providers.picnic_credentials().unwrap().auth_token, "Picnic Login Complete")
        }

        let db_providers = crate::db::providers::all_db_providers(&sea_db).await?;

        let result = Application {
            tcp,
            config: Arc::new(ArcSwap::from_pointee(config)),
            db: sea_db,
            providers: providers.into(),
            db_providers,
        };

        Ok(result)
    }

    /// Start running the Axum server, consuming `Application`.
    /// The future completes when the Tokio-Runtime has been shut down (due to f.e a SIGINT).
    ///
    /// # Arguments
    ///
    /// * `quitter` - A way to inform the spawned runtime to shut down. Especially useful for tests
    /// where we won't provide a signal for shutdown.
    pub async fn run(self, quitter: Arc<tokio::sync::Notify>) -> anyhow::Result<()> {
        tracing::info!("Setup complete, starting server...");
        let app = construct_server(self.db, self.config.clone(), self.providers.clone(), self.db_providers).await?;
        tracing::info!("Listening on {:?}", self.tcp);
        let server = axum::Server::from_tcp(self.tcp)?.serve(app.into_make_service());

        let result = tokio::select! {
            _ = quitter.notified() => Ok(()),
            res = tokio::signal::ctrl_c() => {
                tracing::trace!("Received CTRL-C notification, exiting...");
                // Should notify all dependant sub-processes.
                quitter.notify_waiters();
                res.map_err(|e| anyhow::anyhow!(e))
            },
            res = server => res.map_err(|e| anyhow::anyhow!(e))
        };

        // Persist data cache
        caching::teardown_cache(self.providers.serialized_cache(), &self.config.load()).await;

        result
    }

    pub fn pool(&self) -> DatabaseConnection {
        self.db.clone()
    }
}

async fn construct_server(
    db: DatabaseConnection,
    config: SharedConfig,
    providers: Arc<WggProvider>,
    db_providers: BTreeMap<Provider, Id>,
) -> anyhow::Result<axum::Router> {
    let cfg: DynGuard<Config> = config.load();
    let secret_key = tower_cookies::Key::from(cfg.app.cookie_secret_key.as_bytes());

    let state = State {
        db,
        config,
        providers,
        db_providers,
    };
    let schema = create_graphql_schema(state.clone(), secret_key.clone());

    let app = api_router(&cfg.app.static_dir, schema.clone()).layer(
        ServiceBuilder::new()
            .layer(AddExtensionLayer::new(schema))
            .layer(AddExtensionLayer::new(state))
            .layer(AddExtensionLayer::new(secret_key))
            .layer(TraceLayer::new_for_http())
            .layer(CompressionLayer::new().br(true).gzip(true).deflate(true))
            .layer(CookieManagerLayer::new()),
    );

    Ok(app)
}

fn create_graphql_schema(state: State, secret_key: tower_cookies::Key) -> crate::api::WggSchema {
    Schema::build(
        crate::api::QueryRoot::default(),
        crate::api::MutationRoot::default(),
        EmptySubscription,
    )
    .data(DataLoaders::new())
    .data(state)
    .data(secret_key)
    .extension(crate::api::ErrorTraceExtension)
    .limit_depth(50)
    .finish()
}

fn api_router(static_dir: &Path, schema: crate::api::WggSchema) -> axum::Router {
    let error_handler = |_| std::future::ready(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    let spa_handler = ServeDir::new(static_dir).fallback(ServeFile::new(static_dir.join("index.html")));
    let assets_service: MethodRouter<Body> = get_service(spa_handler).handle_error(error_handler);

    axum::Router::new()
        .nest("/api", crate::api::config(schema))
        .fallback(assets_service)
}

async fn initialise_database(db_cfg: &DbConfig) -> anyhow::Result<SqlitePool> {
    std::fs::create_dir_all(db_cfg.db_path.parent().unwrap())?;

    let options = db_cfg
        .database_url()
        .parse::<SqliteConnectOptions>()?
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal) // Since we're in WAL mode
        .pragma("wal_autocheckpoint", "1000")
        .busy_timeout(Duration::from_secs(10));

    let pool = SqlitePoolOptions::new()
        .max_connections(std::thread::available_parallelism()?.get() as u32)
        .connect_with(options)
        .await?;

    Ok(pool)
}

async fn setup_db_schema(db: &SqlitePool) -> anyhow::Result<()> {
    tracing::info!("Running server database migrations");

    sqlx::migrate!("../migrations")
        .run(db)
        .await
        .context("Error running database migrations")?;

    tracing::info!("Completed server database setup");

    Ok(())
}
