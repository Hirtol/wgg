use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct PicnicCredentials {
    email: String,
    password: SecretString,
    auth_token: Option<SecretString>,
}

impl PicnicCredentials {
    /// Create a new set of credentials.
    ///
    /// If `auth_token` is provided then the initial log-in attempt is skipped when creating the [PicnicBridge].
    pub fn new(email: impl Into<String>, password: SecretString, auth_token: Option<SecretString>) -> Self {
        Self {
            email: email.into(),
            password,
            auth_token,
        }
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn password(&self) -> &secrecy::SecretString {
        &self.password
    }

    pub(crate) fn to_credentials(&self) -> Option<wgg_picnic::Credentials> {
        Some(wgg_picnic::Credentials::new(
            self.auth_token.as_ref()?.expose_secret().clone(),
            "1".to_string(),
        ))
    }
}
