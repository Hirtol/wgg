use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, async_graphql::Enum, Hash, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Provider {
    Picnic,
    Jumbo,
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
