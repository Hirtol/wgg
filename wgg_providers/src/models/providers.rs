use crate::providers::StaticProviderInfo;
use crate::providers::{JumboBridge, PicnicBridge};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::str::FromStr;

pub fn provider_info(provider: Provider) -> ProviderInfo {
    ProviderInfo {
        provider,
        logo_url: provider.get_logo_url(),
    }
}

#[derive(Serialize, Deserialize, async_graphql::Enum, Hash, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Provider {
    Picnic,
    Jumbo,
}

impl Provider {
    /// Get the logo url of the given provider.
    pub fn get_logo_url(&self) -> Cow<'static, str> {
        match self {
            Provider::Picnic => PicnicBridge::logo_url(),
            Provider::Jumbo => JumboBridge::logo_url(),
        }
    }

    /// Retrieve a ProviderInfo object which contains the logo url for the current provider.
    pub fn as_provider_info(&self) -> ProviderInfo {
        ProviderInfo {
            provider: *self,
            logo_url: self.get_logo_url(),
        }
    }
}

impl FromStr for Provider {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PICNIC" => Ok(Provider::Picnic),
            "JUMBO" => Ok(Provider::Jumbo),
            _ => anyhow::bail!("Failed to parse provider {}", s),
        }
    }
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Hash, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProviderInfo {
    /// The grocery store which provided this item.
    pub provider: Provider,
    /// The SVG logo of the grocery store
    pub logo_url: std::borrow::Cow<'static, str>,
}
