use reqwest::Url;
use std::fmt::{Display, Formatter};

pub struct Config {
    pub(crate) country_code: CountryCode,
    pub(crate) url: Url,
    pub(crate) api_version: u16,
    pub(crate) user_argent: String,
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
            country_code,
            url: format!(
                "https://storefront-prod.{}.picnicinternational.com/api/{}",
                country_code, api_version
            )
            .parse()
            .expect("Default URL Incorrect"),
            api_version,
            user_argent: "okhttp/3.12.2".to_string(),
        }
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
