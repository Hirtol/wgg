use crate::providers::StaticProviderInfo;
use crate::providers::{JumboBridge, PicnicBridge};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::str::FromStr;

pub fn provider_info(provider: Provider) -> ProviderInfo {
    ProviderInfo {
        provider,
        logo_url: provider.get_metadata().logo_url,
    }
}

#[derive(Serialize, Deserialize, async_graphql::Enum, Hash, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Provider {
    Picnic,
    Jumbo,
}

impl Provider {
    /// Get all relevant metadata for this provider
    pub fn get_metadata(&self) -> ProviderMetadata {
        match self {
            Provider::Picnic => PicnicBridge::metadata(),
            Provider::Jumbo => JumboBridge::metadata(),
        }
    }

    /// Retrieve a ProviderInfo object which contains the logo url for the current provider.
    pub fn as_provider_info(&self) -> ProviderInfo {
        ProviderInfo {
            provider: *self,
            logo_url: self.get_metadata().logo_url,
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
    pub logo_url: Cow<'static, str>,
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Hash, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProviderMetadata {
    /// The SVG logo of the grocery store
    pub logo_url: Cow<'static, str>,
    /// The strategy for multi-product sale resolution.
    pub sale_strategy: SaleResolutionStrategy,
    /// Whether the provider supports managing cart contents to whatever backend it uses.
    pub supports_cart: bool,
}

/// When resolving multi-product (think `1 + 1 free`, `2nd half off`, etc) the strategy determines how products will
/// be grouped.
///
/// Specifically, this is for cases when -- in the case of a `1 + 1 free` deal -- one has `3` qualifying products of varying
/// prices.
#[derive(Serialize, Deserialize, async_graphql::Enum, Hash, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SaleResolutionStrategy {
    /// When resolving a multi-product sale the provider will ensure the largest savings for the customer.
    Opportunistic,
    /// When resolving a multi-product sale the provider will ensure the smallest savings for the customer.
    Pessimistic,
}
