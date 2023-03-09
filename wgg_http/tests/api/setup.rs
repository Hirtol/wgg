#![allow(dead_code)]
use sea_orm::DatabaseConnection;
use std::sync::Arc;

use reqwest::{ClientBuilder, Method, RequestBuilder};
use tempfile::TempDir;
use tracing_subscriber::util::SubscriberInitExt;
use wgg_http::config::Config;

use wgg_http::setup::{Application, DEFAULT_USER};
use wgg_http::telemetry;

use crate::graphql::{post_graphql_request, GraphQLCustomRequest, GraphQLCustomResponse};

pub type ApiRoute = str;
pub type Id = u64;

pub struct TestSettings {
    pub config: Config,
}

#[derive(Debug, Clone)]
pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db_pool: DatabaseConnection,
    pub temp_dir: Arc<TempDir>,
    quit_notifier: Arc<tokio::sync::Notify>,
}

impl TestApp {
    pub async fn spawn_app() -> Self {
        Self::spawn_app_with_settings(Self::settings()).await
    }

    pub async fn spawn_app_with_settings(mut test_settings: TestSettings) -> Self {
        // Setup Tracing
        let subscriber = telemetry::create_subscriber("DEBUG,wgg_http=TRACE,wgg_providers=TRACE,sqlx=WARN,hyper=WARN");
        let _ = subscriber.try_init();

        // Spawn the actual app
        let temp_dir = Arc::new(tempfile::tempdir().expect("Couldn't create a temp appdata directory!"));
        test_settings.config.app.cache_dir = temp_dir.path().join("cache");

        let app = Application::new(test_settings.config)
            .await
            .expect("Failed to construct a TestApp for a test!");

        let port = app.port().local_addr().unwrap().port();
        let pool = app.pool();
        let quit_notifier = wgg_http::get_quit_notifier();

        let _ = tokio::spawn(app.run(quit_notifier.clone()));

        TestApp {
            address: format!("http://localhost:{port}"),
            port,
            db_pool: pool,
            temp_dir,
            quit_notifier,
        }
    }

    pub fn settings() -> TestSettings {
        let mut settings = Config::default();
        settings.app.port = 0;
        settings.app.startup_sale_validation = false;
        settings.db.in_memory = true;

        TestSettings { config: settings }
    }

    pub fn into_client(self) -> WggClient {
        WggClient::new(self)
    }

    pub async fn into_authenticated_client(self) -> WggClient {
        WggClient::with_login(self).await
    }

    pub fn address_route(&self, route: impl AsRef<str>) -> String {
        format!("{}{}", self.address, route.as_ref())
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        self.quit_notifier.notify_waiters()
    }
}

pub struct WggClient {
    pub client: reqwest::Client,
    pub app: TestApp,
}

impl WggClient {
    pub fn new(app: TestApp) -> Self {
        WggClient {
            client: ClientBuilder::new()
                .cookie_store(true)
                .build()
                .expect("Failed to create a Reqwest client!"),
            app,
        }
    }

    /// Create a client and log in with the admin user immediately.
    pub async fn with_login(app: TestApp) -> Self {
        Self::with_login_and_user_id(app).await.0
    }

    /// Create a client and log in with the admin user, returning the user's ID.
    pub async fn with_login_and_user_id(app: TestApp) -> (Self, Id) {
        let result = Self::new(app);
        let response = result.login(&DEFAULT_USER.email, &DEFAULT_USER.password).await;
        (result, response.unwrap())
    }

    /// Log in as the provided user
    pub async fn login(&self, email: &str, password: &str) -> Option<Id> {
        //language=GraphQL
        let query = "
        mutation login($email: String!, $password: String!){
            login(input: {email: $email, password: $password}) {
                user {
                    id
                }
            }
        }
        ";

        let req = GraphQLCustomRequest::from_query(query)
            .with_variable("email", email)
            .with_variable("password", password);

        let response = self.graphql_request(req).await.unwrap();

        response.data["login"]["user"]["id"].as_u64()
    }

    /// Send a GraphQL request to the server.
    pub async fn graphql_request(&self, request: GraphQLCustomRequest) -> anyhow::Result<GraphQLCustomResponse> {
        post_graphql_request(self, request).await
    }

    pub fn request(&self, route: impl AsRef<ApiRoute>, mode: Method) -> RequestBuilder {
        self.client.request(mode, self.app.address_route(route))
    }

    pub fn get(&self, route: impl AsRef<ApiRoute>) -> RequestBuilder {
        self.client.get(self.app.address_route(route))
    }

    pub fn post(&self, route: impl AsRef<ApiRoute>) -> RequestBuilder {
        self.client.post(self.app.address_route(route))
    }

    pub fn put(&self, route: impl AsRef<ApiRoute>) -> RequestBuilder {
        self.client.put(self.app.address_route(route))
    }

    pub fn patch(&self, route: impl AsRef<ApiRoute>) -> RequestBuilder {
        self.client.patch(self.app.address_route(route))
    }

    pub fn delete(&self, route: impl AsRef<ApiRoute>) -> RequestBuilder {
        self.client.delete(self.app.address_route(route))
    }
}
