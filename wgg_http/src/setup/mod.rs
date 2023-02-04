use crate::api::dataloader::DataLoaders;
use crate::api::State;
use crate::config::{Config, DbConfig, SharedConfig};
use crate::db::Id;
use anyhow::Context;
use arc_swap::access::{DynAccess, DynGuard};
use arc_swap::ArcSwap;
use async_graphql::{EmptySubscription, Schema};
use axum::body::Body;
use axum::error_handling::HandleErrorLayer;
use axum::http::{header, HeaderValue, StatusCode};
use axum::routing::{get_service, MethodRouter};
use axum::{BoxError, Router};
use sea_orm::{DatabaseConnection, SqlxSqliteConnector};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::SqlitePool;
use std::collections::BTreeMap;
use std::net::TcpListener;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tower::{Layer, ServiceBuilder};
use tower_cookies::CookieManagerLayer;
use tower_http::add_extension::AddExtensionLayer;
use tower_http::compression::CompressionLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use wgg_providers::models::Provider;
use wgg_providers::WggProvider;

pub use first_time::DEFAULT_USER;
use wgg_scheduler::JobScheduler;

mod caching;
mod first_time;

pub struct Application {
    pub tcp: TcpListener,
    pub config: SharedConfig,
    pub db: DatabaseConnection,
    pub providers: Arc<WggProvider>,
    pub db_providers: BTreeMap<Provider, Id>,
    pub scheduler: JobScheduler,
}

impl Application {
    #[tracing::instrument(name = "Create application", skip(config), fields(addr = config.app.host, port = config.app.port))]
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let tcp = TcpListener::bind(config.app.bind_address())?;
        let db = initialise_database(&config.db).await?;

        setup_db_schema(&db).await?;

        let sea_db = SqlxSqliteConnector::from_sqlx_sqlite_pool(db);
        first_time::first_time_setup(&sea_db).await?;

        tracing::debug!("Creating Providers...");
        let cache = caching::setup_cache(&config).await;
        let mut providers_builder = WggProvider::builder()
            .with_cache(cache)
            .with_startup_sale_validation(config.app.startup_sale_validation)
            .with_jumbo(Default::default())
            .with_picnic_rps(Some(config.pd.picnic.requests_per_second));

        // Try initialise the Picnic provider.
        match config.pd.picnic.clone().try_into() {
            Ok(picnic_creds) => {
                providers_builder = providers_builder.with_picnic(picnic_creds);
            }
            Err(e) => tracing::debug!(error = %e, "Not using Picnic Provider"),
        }

        let providers = providers_builder.build().await?;

        let scheduler = JobScheduler::new(Duration::from_millis(500)).await;

        let db_providers = crate::db::providers::all_db_providers(&sea_db).await?;

        let result = Application {
            tcp,
            config: Arc::new(ArcSwap::from_pointee(config)),
            db: sea_db,
            providers: providers.into(),
            db_providers,
            scheduler,
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
    pub async fn run(self, quitter: Arc<tokio::sync::Notify>) -> anyhow::Result<Arc<Config>> {
        tracing::info!("Setup complete, starting server...");

        self.scheduler.start().await;

        let app = construct_server(
            self.db,
            self.config.clone(),
            self.providers.clone(),
            self.db_providers,
            self.scheduler.clone(),
        )
        .await?;

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
        let mut cfg = self.config.load_full();
        let final_config = Arc::<Config>::make_mut(&mut cfg);

        final_config.pd.picnic.auth_token = self.providers.picnic_auth_token().await;
        caching::teardown_cache(self.providers.serialized_cache(), final_config).await;
        self.scheduler.stop().await?;

        result.map(|_| cfg)
    }

    pub fn pool(&self) -> DatabaseConnection {
        self.db.clone()
    }

    pub fn port(&self) -> &TcpListener {
        &self.tcp
    }
}

async fn construct_server(
    db: DatabaseConnection,
    config: SharedConfig,
    providers: Arc<WggProvider>,
    db_providers: BTreeMap<Provider, Id>,
    scheduler: JobScheduler,
) -> anyhow::Result<axum::Router> {
    let cfg: DynGuard<Config> = config.load();
    let secret_key = tower_cookies::Key::from(cfg.app.cookie_secret_key.as_bytes());

    let state = State {
        db,
        config,
        providers,
        scheduler,
        db_providers,
    };
    let schema = create_graphql_schema(state.clone(), secret_key.clone());

    // Schedule all API jobs
    state.providers.clone().schedule_all_jobs(&state.scheduler);
    crate::api::scheduled_jobs::schedule_all_jobs(&state.scheduler, state.clone());

    let app_layers = ServiceBuilder::new()
        .layer(AddExtensionLayer::new(schema.clone()))
        .layer(AddExtensionLayer::new(state))
        .layer(AddExtensionLayer::new(secret_key))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new().br(true).gzip(true).deflate(true))
        .layer(CookieManagerLayer::new());

    let app = api_router(schema, &cfg.app.static_dir, &cfg.security).layer(app_layers);

    Ok(apply_security_middleware(app, &cfg))
}

fn create_graphql_schema(state: State, secret_key: tower_cookies::Key) -> crate::api::WggSchema {
    Schema::build(
        crate::api::QueryRoot::default(),
        crate::api::MutationRoot::default(),
        EmptySubscription,
    )
    .data(DataLoaders::new(state.db.clone()))
    .data(state)
    .data(secret_key)
    .extension(crate::api::ErrorTraceExtension)
    .limit_depth(50)
    .finish()
}

fn api_router(schema: crate::api::WggSchema, static_dir: &Path, security: &crate::config::Security) -> axum::Router {
    let error_handler = |_| std::future::ready(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    let spa_handler = ServeDir::new(static_dir).fallback(ServeFile::new(static_dir.join("index.html")));
    let assets_service: MethodRouter<Body> = get_service(spa_handler).handle_error(error_handler);
    let assets_service = apply_static_security_headers(assets_service, security);

    // For some reason manifest.json isn't picked up in ServeDir, so we have to special case it here.
    axum::Router::new()
        .route(
            "/manifest.json",
            get_service(ServeFile::new(static_dir.join("manifest.json"))).handle_error(error_handler),
        )
        .nest("/api", crate::api::config(schema))
        .fallback(assets_service)
}

fn apply_static_security_headers(router: MethodRouter<Body>, security: &crate::config::Security) -> MethodRouter<Body> {
    let security = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(generic_error_handler))
        .option_layer(
            security
                .clickjack_protection
                .then_some(SetResponseHeaderLayer::overriding(
                    header::X_FRAME_OPTIONS,
                    HeaderValue::from_static("SAMEORIGIN"),
                )),
        )
        .layer(SetResponseHeaderLayer::overriding(
            header::X_XSS_PROTECTION,
            HeaderValue::from_static("1; mode=block"),
        ));

    router.layer(security)
}

fn apply_security_middleware(router: Router, cfg: &Config) -> Router {
    let security = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(generic_error_handler))
        .load_shed()
        .concurrency_limit(cfg.security.max_concurrency as usize)
        .layer(tower_http::timeout::TimeoutLayer::new(cfg.security.timeout));

    router.layer(security)
}

async fn generic_error_handler(_error: BoxError) -> impl axum::response::IntoResponse {
    tracing::trace!(error=?_error, "Error occurred in normal respone handler");
    (StatusCode::INTERNAL_SERVER_ERROR, "Internal Error")
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
        .max_connections(db_cfg.max_connections.get())
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
