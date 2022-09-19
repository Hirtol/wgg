use reqwest::Url;
use std::fmt::{Display, Formatter};

pub struct Config {
    pub(crate) url: Url,
    pub(crate) static_url: Url,
    pub(crate) user_agent: String,
}

impl Default for Config {
    fn default() -> Self {
        Self::new(CountryCode::NL, 17)
    }
}

impl Config {
    /// A [Config] instance for configuring the [crate::PicnicApi].
    ///
    /// Current defaults are [CountryCode::NL], and `api_version = 17`.
    pub fn new(country_code: CountryCode, api_version: u16) -> Self {
        Config {
            url: format!(
                "https://storefront-prod.{}.picnicinternational.com/api/{}",
                country_code, api_version
            )
            .parse()
            .expect("Default URL Incorrect"),
            static_url: format!(
                "https://storefront-prod.{}.picnicinternational.com/static",
                country_code
            )
            .parse()
            .expect("Default URL Incorrect"),
            user_agent: "okhttp/3.12.2".to_string(),
        }
    }

    /// Returns the API url.
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Returns the URL for accessing static (unauthorised) content.
    pub fn static_url(&self) -> &Url {
        &self.static_url
    }

    /// Returns the full url for the API and the provided suffix.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use wgg_picnic::Config;
    /// let config = Config::default();
    ///
    /// assert_eq!(config.get_full_url("/cart"), "https://storefront-prod.nl.picnicinternational.com/api/17/cart")
    /// ```
    pub fn get_full_url(&self, suffix: &str) -> String {
        format!("{}{}", self.url(), suffix)
    }
}

#[derive(Debug, Clone, Copy, Hash, serde::Serialize, serde::Deserialize)]
pub enum CountryCode {
    NL,
    DE,
}

impl Display for CountryCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let country_code = match self {
            CountryCode::NL => "NL",
            CountryCode::DE => "DE",
        };

        f.write_str(country_code)?;

        Ok(())
    }
}
