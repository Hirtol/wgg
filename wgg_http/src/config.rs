use anyhow::anyhow;
use arc_swap::ArcSwap;
use figment::providers::{Env, Format, Serialized, Toml};
use figment::Figment;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use std::io::Write;
use std::net::ToSocketAddrs;
use std::path::PathBuf;
use std::sync::Arc;

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

#[derive(Serialize, Deserialize, Clone, Hash, PartialOrd, PartialEq, Eq, Default)]
#[serde(default)]
pub struct Config {
    /// Contains all settings related to app hosting/config values.
    pub app: AppConfig,
    /// Contains all settings relevant for DB initialisation.
    pub db: DbConfig,
    /// Contains all settings relevant for authentication with external services.
    #[serde(skip_serializing)]
    pub auth: AuthConfig,
}

impl Debug for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("app", &self.app)
            .field("db", &self.db)
            .finish()
    }
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
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialOrd, PartialEq, Eq)]
#[serde(default)]
pub struct DbConfig {
    /// Full path to the DB file.
    pub db_path: PathBuf,
    pub in_memory: bool,
}

#[derive(Deserialize, Clone, Hash, PartialOrd, PartialEq, Eq, Default)]
pub struct AuthConfig {
    pub picnic_auth_token: Option<String>,
    pub picnic_username: Option<String>,
    pub picnic_password: Option<String>,
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
        }
    }
}

impl Default for DbConfig {
    fn default() -> Self {
        DbConfig {
            db_path: crate::utils::get_app_dirs().config_dir.join("wgg.db"),
            in_memory: false,
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
