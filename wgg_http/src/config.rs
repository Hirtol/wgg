use anyhow::anyhow;
use arc_swap::ArcSwap;
use figment::providers::{Env, Format, Serialized, Toml};
use figment::Figment;
use rand::Rng;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use std::io::Write;
use std::net::ToSocketAddrs;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

pub type SharedConfig = Arc<ArcSwap<Config>>;

static CONFIG_FILE: &str = "config.toml";

/// Initialise the config file.
///
/// Creates a new config file if it doesn't yet exist, otherwise loads the existing one.
pub fn initialise_config() -> anyhow::Result<Config> {
    let c_path = get_full_config_path();

    if !c_path.exists() {
        save_config(&Config::default())?;
    }

    let result = Figment::from(Serialized::defaults(Config::default()))
        .merge(Toml::file(&c_path))
        .merge(get_environment_provider())
        .extract()
        .map_err(|e| anyhow!(e))?;

    // For the initial config creation, pass the obtained environment variables to ensure consistency.
    save_config(&result)?;

    Ok(result)
}

/// Save the provided config to the known config directory.
pub fn save_config(app_settings: &Config) -> anyhow::Result<()> {
    std::fs::create_dir_all(get_config_directory())?;

    let mut config_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(get_full_config_path())?;

    let basic_output = toml::to_string_pretty(app_settings)?;

    config_file.write_all(basic_output.as_bytes())?;

    Ok(())
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Config {
    /// Contains all settings related to app hosting/config values.
    pub app: AppConfig,
    /// Contains all settings relevant for DB initialisation.
    pub db: DbConfig,
    /// Contains all settings relevant for the various providers.
    pub pd: ProviderConfig,
    /// Contains all settings relevant for security/rate limiting.
    pub security: Security,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Security {
    /// Whether to block the embedding of the application in IFrames.
    ///
    /// (Sets x-frame header)
    pub clickjack_protection: bool,
    /// The maximum amount of requests concurrently being handled.
    pub max_concurrency: u32,
    /// How long a request can take before automatically being timed out.
    pub timeout: Duration,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialOrd, PartialEq, Eq)]
#[serde(default)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    /// The secret key used to encrypt session cookies. This should remain private.
    pub cookie_secret_key: String,
    /// The directory where the front-end is located.
    pub static_dir: PathBuf,
    /// The directory where the provider product cache is stored between runs
    pub cache_dir: PathBuf,
    /// On startup immediately fetch the latest sales and compare them with the local cache.
    ///
    /// Will be done asynchronously but will send *a lot* (~80 per provider) of requests.
    pub startup_sale_validation: bool,
    /// Whether to enable the GraphQL playground.
    ///
    /// Disabled by default.
    pub graphql_playground: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialOrd, PartialEq, Eq)]
#[serde(default)]
pub struct DbConfig {
    /// Full path to the DB file.
    pub db_path: PathBuf,
    pub in_memory: bool,
    /// The amount of connections to the database.
    /// In theory Sqlx should've fixed 'database locked' errors, but those still seem to occur with multiple mutations.
    /// Hence why we use just a single connection by default.
    pub max_connections: NonZeroU32,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct ProviderConfig {
    pub picnic: PicnicConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PicnicConfig {
    /// The maximum requests per second to allow towards Picnic servers.
    /// More is better, but comes with a greater risk of API bans.
    pub requests_per_second: NonZeroU32,
    /// The email associated with the Picnic account.
    ///
    /// Both the email and password should be provided through environment variables
    #[serde(skip_serializing)]
    pub picnic_email: Option<String>,
    #[serde(skip_serializing)]
    pub picnic_password: Option<SecretString>,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            cookie_secret_key: rand::thread_rng()
                .sample_iter(rand::distributions::Alphanumeric)
                .take(128)
                .map(char::from)
                .collect(),
            static_dir: std::env::current_dir()
                .expect("Can't get current directory")
                .join("static"),
            cache_dir: crate::utils::get_app_dirs().config_dir.join("cache"),
            startup_sale_validation: true,
            graphql_playground: false,
        }
    }
}

impl Debug for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("app", &self.app)
            .field("db", &self.db)
            .field("pd", &self.pd)
            .finish()
    }
}

impl Default for Security {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_concurrency: 512,
            clickjack_protection: true,
        }
    }
}

impl Default for DbConfig {
    fn default() -> Self {
        DbConfig {
            db_path: crate::utils::get_app_dirs().config_dir.join("wgg.db"),
            in_memory: false,
            max_connections: std::thread::available_parallelism()
                .ok()
                .and_then(|x| x.try_into().ok())
                .unwrap_or_else(|| NonZeroU32::new(1).unwrap()),
        }
    }
}

impl AppConfig {
    /// Turn the app config settings into a [ToSocketAddrs]
    pub fn bind_address(&self) -> impl ToSocketAddrs {
        (self.host.clone(), self.port)
    }
}

impl DbConfig {
    /// Turn the config settings into a valid DB url.
    pub fn database_url(&self) -> String {
        if self.in_memory {
            "sqlite::memory:".to_string()
        } else {
            format!(
                "sqlite://{}?mode=rwc",
                self.db_path
                    .to_str()
                    .expect("Invalid database path specified in config or ENV")
            )
        }
    }
}

impl Default for PicnicConfig {
    fn default() -> Self {
        Self {
            requests_per_second: wgg_providers::PICNIC_RECOMMENDED_RPS.unwrap(),
            picnic_email: None,
            picnic_password: None,
        }
    }
}

impl TryFrom<PicnicConfig> for wgg_providers::PicnicCredentials {
    type Error = anyhow::Error;

    fn try_from(value: PicnicConfig) -> Result<Self, Self::Error> {
        let (Some(email), Some(password)) = (value.picnic_email, value.picnic_password) else {
            anyhow::bail!("Either the email or password was missing for Picnic Credentials initialisation");
        };

        Ok(Self::new(email, password))
    }
}

/// Retrieve the *full* path to the config file.
///
/// This is just [get_config_directory] + [CONFIG_FILE]
pub fn get_full_config_path() -> PathBuf {
    get_config_directory().join(CONFIG_FILE)
}

/// Retrieve the directory which will be used to locate/save the config file.
pub fn get_config_directory() -> PathBuf {
    get_environment_provider()
        .extract_inner("appdata_dir")
        .unwrap_or_else(|_| crate::utils::get_app_dirs().config_dir)
}

fn get_environment_provider() -> Figment {
    // This rather hacky workaround is needed to make variables using `_` work. (otherwise we'd just split on `_`)
    Figment::from(Env::prefixed("WGG__").split("__"))
}
