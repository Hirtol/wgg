use std::sync::Arc;

use anyhow::{anyhow, Context};
use futures::future::BoxFuture;
use futures::FutureExt;
use md5::Digest;
use reqwest::header::HeaderMap;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::models::{LoginRequest, LoginResponse};
use crate::{get_reqwest_client, ApiError, Config};

#[derive(Deserialize, Clone, Serialize, Debug)]
pub struct Credentials {
    pub auth_token: String,
    pub user_id: String,
}

impl Credentials {
    pub fn new(auth_token: String, user_id: String) -> Self {
        Self { auth_token, user_id }
    }
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct LoginCredentials {
    pub email: String,
    pub password: String,
}

pub struct CredentialsManager {
    config: Arc<Config>,
    login_credentials: LoginCredentials,
    cache: Box<RwLock<dyn CredentialsCache>>,
    refresh_lock: tokio::sync::Mutex<()>,
}

impl CredentialsManager {
    pub fn new(credentials_cache: impl CredentialsCache, config: Arc<Config>, login: LoginCredentials) -> Self {
        Self {
            config,
            login_credentials: login,
            cache: Box::new(RwLock::new(credentials_cache)),
            refresh_lock: tokio::sync::Mutex::new(()),
        }
    }

    pub async fn credentials(&self) -> crate::Result<Arc<Credentials>> {
        let credentials = self.cache.read().await.request_credentials().await?;

        match credentials {
            Some(creds) => Ok(creds),
            _ => {
                // Need to refresh the credentials.
                // Probably better done with a separate task, but this ensures only a single task actually does the refresh.
                match self.refresh_lock.try_lock() {
                    Ok(_lock) => self.force_refresh().await,
                    Err(_) => {
                        // Wait for the refresh to occur above on a different task
                        drop(self.refresh_lock.lock().await);
                        // If the above failed, then we'll just error out here and let the caller take care of it.
                        self.cache
                            .read()
                            .await
                            .request_credentials()
                            .await?
                            .ok_or(ApiError::LoginFailed("Failed to aqcuire new credentials!".to_string()))
                    }
                }
            }
        }
    }

    /// Handle a response, and update the JWT.
    pub async fn handle_response(&self, response: &reqwest::Response) -> crate::Result<()> {
        match response.status() {
            StatusCode::OK => {
                if let Some(auth_header) = response.headers().get("x-picnic-auth") {
                    let auth_token = auth_header.to_str().context("Failed to convert to str")?.to_string();

                    self.cache.write().await.update_auth_token(auth_token).await?;
                }

                Ok(())
            }
            StatusCode::UNAUTHORIZED => {
                tracing::debug!(status = %response.status(), ?response, "Picnic API Error");
                Err(ApiError::AuthError)
            }
            _ => Ok(()),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn force_refresh(&self) -> crate::Result<Arc<Credentials>> {
        let new_credentials = Arc::new(
            self.fetch_credentials(self.login_credentials.email.clone(), &self.login_credentials.password)
                .await?,
        );
        self.cache
            .write()
            .await
            .persist_credentials(new_credentials.clone())
            .await?;

        Ok(new_credentials)
    }

    #[tracing::instrument(skip(self))]
    async fn fetch_credentials(&self, email: String, password: &str) -> crate::Result<Credentials> {
        let mut hasher = md5::Md5::new();

        hasher.update(password);

        let result = hasher.finalize();
        let hex = hex::encode(result);

        let client = get_reqwest_client(&self.config.user_agent)?;
        let login = LoginRequest {
            key: email,
            secret: hex,
            client_id: self
                .config
                .picnic_details
                .as_ref()
                .map(|i| i.client_id.clone())
                .unwrap_or_else(|| "30100".to_string()),
            client_version: self.config.picnic_details.as_ref().map(|i| i.client_version.clone()),
            device_id: None,
        };

        let response = client
            .post(self.config.get_full_url("/user/login"))
            .json(&login)
            .send()
            .await?;

        if response.status().is_client_error() {
            return Err(ApiError::LoginFailed(format!(
                "Status: {} - Body: {}",
                response.status(),
                response.text().await?
            )));
        }

        let mut auth_token = response
            .headers()
            .get("x-picnic-auth")
            .ok_or_else(|| anyhow!("No picnic auth token available in response: {:#?}", response))?
            .to_str()
            .context("Failed to convert to str")?
            .to_string();
        let login_response: LoginResponse = response.json().await?;

        // Need to send a sms and request the credentials.
        if login_response.second_factor_authentication_required {
            tracing::debug!("Starting second factor Picnic verification");
            let mut headers = HeaderMap::new();
            headers.insert(
                "x-picnic-auth",
                (&auth_token).try_into().context("Failed to convert agent")?,
            );
            if let Some(agent) = self.config.picnic_agent() {
                headers.insert("x-picnic-agent", agent.try_into().context("Failed to convert agent")?);
            }

            // First send an SMS
            let response = client
                .post(self.config.get_full_url("/user/2fa/generate"))
                .headers(headers.clone())
                .json(&serde_json::json!({
                    "channel": "SMS"
                }))
                .send()
                .await?;

            if !response.status().is_success() {
                return Err(ApiError::LoginFailed("Failed to send a 2fa request".to_string()));
            }

            let fa_code = self.cache.read().await.request_2fa_code().await?;
            tracing::debug!(fa_code, "Sending 2FA code");

            let response = client
                .post(self.config.get_full_url("/user/2fa/verify"))
                .headers(headers)
                .json(&serde_json::json!({
                    "otp": fa_code
                }))
                .send()
                .await?;

            if !response.status().is_success() {
                return Err(ApiError::LoginFailed("Failed to verify a 2fa code".to_string()));
            }

            auth_token = response
                .headers()
                .get("x-picnic-auth")
                .ok_or_else(|| anyhow!("No picnic auth token available in response: {:#?}", response))?
                .to_str()
                .context("Failed to convert to str")?
                .to_string();
        }

        Ok(Credentials {
            auth_token,
            user_id: login_response.user_id,
        })
    }
}

pub trait CredentialsCache: Send + Sync + 'static {
    /// Request the currently cached credentials, note that this will be called on each request,
    /// so it should be cheap to execute.
    ///
    /// The caller must check for expiration.
    ///
    /// # Returns
    ///
    /// `Some(credentials)` if the credentials exist, `None` if they don't exist and should be re-acquired, or `Err` if something went wrong.
    fn request_credentials(&self) -> BoxFuture<crate::Result<Option<Arc<Credentials>>>>;

    /// Persist the newly acquired credentials in the cache.
    fn persist_credentials(&mut self, credentials: Arc<Credentials>) -> BoxFuture<crate::Result<()>>;

    /// Update just the auth token.
    fn update_auth_token(&mut self, auth_token: String) -> BoxFuture<crate::Result<()>> {
        async move {
            let Some(creds) = self.request_credentials().await?.take() else {
                return Ok(());
            };

            self.persist_credentials(Arc::new(Credentials::new(auth_token, creds.user_id.clone())))
                .await
        }
        .boxed()
    }

    /// Request the user to provide a 2fa code
    fn request_2fa_code(&self) -> BoxFuture<crate::Result<String>>;
}

pub mod cache {
    use std::io::BufWriter;
    use std::path::PathBuf;
    use std::sync::Arc;

    use futures::future::BoxFuture;
    use futures::FutureExt;

    use crate::credentials::Credentials;
    use crate::ApiError;

    pub struct MemoryCache {
        current_credentials: Option<Arc<Credentials>>,
    }

    impl MemoryCache {
        /// Create a new ephemeral in-memory cache.
        pub fn new(credentials: Option<Arc<Credentials>>) -> Self {
            MemoryCache {
                current_credentials: credentials,
            }
        }
    }

    impl super::CredentialsCache for MemoryCache {
        fn request_credentials(&self) -> BoxFuture<crate::Result<Option<Arc<Credentials>>>> {
            async { Ok(self.current_credentials.clone()) }.boxed()
        }

        fn persist_credentials(&mut self, credentials: Arc<Credentials>) -> BoxFuture<crate::Result<()>> {
            async move {
                self.current_credentials = Some(credentials.into());
                Ok(())
            }
            .boxed()
        }

        fn request_2fa_code(&self) -> BoxFuture<crate::Result<String>> {
            async move { Err(ApiError::NoSecondFactorCode) }.boxed()
        }
    }

    pub struct JsonFileCache {
        mem_cache: MemoryCache,
        cache_path: PathBuf,
    }

    impl JsonFileCache {
        /// Initialise the cache from the given file.
        ///
        /// If the file doesn't exist, or if there was some other problem, the [Credentials] will be initialised as [None].
        pub fn from_file(file: impl Into<PathBuf>) -> Self {
            let path = file.into();
            let credentials = if let Ok(contents) = std::fs::read(&path) {
                if let Ok(cert) = serde_json::from_slice(&contents) {
                    Some(Arc::new(cert))
                } else {
                    None
                }
            } else {
                None
            };

            Self::from_credentials(path, credentials)
        }

        /// Initialise the cache with the given set of `credentials` and a `file` path.
        ///
        /// Note that `file` is not opened, for loading [Credentials] from `file` see [Self::from_file].
        ///
        /// If [super::CredentialsCache::persist_credentials] is called the given `file` path will be used to save them immediately.
        pub fn from_credentials(file: impl Into<PathBuf>, credentials: Option<Arc<Credentials>>) -> Self {
            Self {
                mem_cache: MemoryCache::new(credentials),
                cache_path: file.into(),
            }
        }
    }

    impl super::CredentialsCache for JsonFileCache {
        fn request_credentials(&self) -> BoxFuture<crate::Result<Option<Arc<Credentials>>>> {
            self.mem_cache.request_credentials()
        }

        fn persist_credentials(&mut self, credentials: Arc<Credentials>) -> BoxFuture<crate::Result<()>> {
            async move {
                if let Some(path) = self.cache_path.parent() {
                    std::fs::create_dir_all(path).map_err(|e| anyhow::anyhow!(e))?;
                }

                // Persist to file, use std::fs for convenience.
                let mut file = BufWriter::new(std::fs::File::create(&self.cache_path).map_err(|e| anyhow::anyhow!(e))?);
                serde_json::to_writer(&mut file, credentials.as_ref()).map_err(|e| anyhow::anyhow!(e))?;

                self.mem_cache.persist_credentials(credentials).await
            }
            .boxed()
        }

        fn request_2fa_code(&self) -> BoxFuture<crate::Result<String>> {
            async move { Err(ApiError::NoSecondFactorCode) }.boxed()
        }
    }
}
