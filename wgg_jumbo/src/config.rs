use reqwest::Url;

pub struct Config {
    pub(crate) url: Url,
    pub(crate) user_agent: String,
}

impl Default for Config {
    fn default() -> Self {
        Self::new(17)
    }
}

impl Config {
    /// A [Config] instance for configuring the [crate::JumboApi].
    ///
    /// Current default is `api_version = 17`.
    pub fn new(api_version: u16) -> Self {
        Config {
            url: format!("https://mobileapi.jumbo.com/v{}", api_version)
                .parse()
                .expect("Default URL Incorrect"),
            user_agent: "Jumbo/9.4.1 (unknown Android_SDK_built_for_x86_64; Android 12)".to_string(),
        }
    }

    /// Returns the API url.
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Returns the full url for the API and the provided suffix.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use wgg_jumbo::Config;
    /// let config = Config::default();
    ///
    /// assert_eq!(config.get_full_url("/categories"), "https://mobileapi.jumbo.com/v17/categories")
    /// ```
    pub fn get_full_url(&self, suffix: &str) -> String {
        format!("{}{}", self.url(), suffix)
    }
}
