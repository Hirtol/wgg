use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct PicnicCredentials {
    email: String,
    password: SecretString,
}

impl PicnicCredentials {
    /// Create a new set of credentials.
    ///
    /// If `auth_token` is provided then the initial log-in attempt is skipped when creating the [PicnicBridge].
    pub fn new(email: impl Into<String>, password: SecretString) -> Self {
        Self {
            email: email.into(),
            password,
        }
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn password(&self) -> &secrecy::SecretString {
        &self.password
    }

    pub(crate) fn to_login(&self) -> wgg_picnic::LoginCredentials {
        wgg_picnic::LoginCredentials {
            email: self.email.clone(),
            password: self.password.expose_secret().clone(),
        }
    }
}
