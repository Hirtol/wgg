use reqwest::Url;
use std::fmt::{Display, Formatter};

pub struct Config {
    pub(crate) url: Url,
    pub(crate) static_url: Url,
    pub(crate) user_agent: String,
    pub(crate) picnic_details: Option<PicnicDetails>,
}

impl Config {
    /// A [Config] instance for configuring the [crate::PicnicApi].
    ///
    /// Current defaults are [CountryCode::NL], `api_version = 17`, and `picnic_details = None`.
    ///
    /// `picnic_details` is optional, when it is provided any API key generated with this active will always need to adhere to this.
    /// Two factor authentication will also need to be called if it is provided.
    pub fn new(country_code: CountryCode, api_version: u16, picnic_details: Option<PicnicDetails>) -> Self {
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
            picnic_details,
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

    /// Returns the Picnic Agent if `picnic_details` was provided during construction in [Self::new].
    pub fn picnic_agent(&self) -> Option<&str> {
        self.picnic_details.as_ref().map(|i| i.picnic_agent.as_str())
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

impl Default for Config {
    fn default() -> Self {
        Self::new(CountryCode::NL, 17, None)
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

/// An optional component of the Picnic API.
///
/// The API functions without this as well, but it provides more complete list information with it present.
/// For example, promotions will have their full list laid out if the `total items < 4`.
///
/// Note that any API key that is generated while these details are provided will only be valid so long as these details
/// are *always* provided.
#[derive(Debug, Clone)]
pub struct PicnicDetails {
    /// A combination of [client_id] and [client_version], looks as follows `30100;1.15.159;`.
    pub picnic_agent: String,
    /// Extracted from a decompiled app is a constant `30100`, may change in the future
    pub client_id: String,
    /// Semver 3 part a.k.a `1.15.159` for example.
    /// In the official app is made up as follows `android:versionName-android:versionCode`, both of which can be
    /// extracted from a decompiled app version.
    ///
    /// For the sake of the API only the Semver part is necessary, however.
    pub client_version: String,
}

impl Default for PicnicDetails {
    fn default() -> Self {
        PicnicDetails {
            picnic_agent: "30100;1.15.159;".to_string(),
            // Extracted from the decompiled app, is just a constant?
            client_id: "30100".to_string(),
            // Can contain a - android:versionCode component as well
            client_version: "1.15.159".to_string(),
        }
    }
}

impl PicnicDetails {
    pub fn new(client_id: impl Into<String>, client_version: impl Into<String>) -> PicnicDetails {
        let client_id = client_id.into();
        let client_version = client_version.into();
        let agent = format!("{};{};", client_id, client_version);

        Self {
            picnic_agent: agent,
            client_id,
            client_version,
        }
    }
}
