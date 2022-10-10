use crate::models::{Provider, ProviderInfo, WggDecorator, WggSearchProduct};
use crate::providers::StaticProviderInfo;
use crate::{JumboBridge, PicnicBridge};
use serde::{Deserialize, Serialize};

// ** Promotions **
#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
#[graphql(complex)]
pub struct WggSaleCategory {
    pub id: String,
    pub name: String,
    pub image_urls: Vec<String>,
    /// A potentially limited selection of items, only supported for certain [Provider]s.
    ///
    /// Picnic is one example of such a provider.
    /// Generally recommended to query for more detailed information when needed.
    pub limited_items: Vec<PromotionProduct>,
    pub decorators: Vec<WggDecorator>,
    #[graphql(skip)]
    pub provider: Provider,
}

#[async_graphql::ComplexObject]
impl WggSaleCategory {
    /// Grocery store information associated with this item
    async fn provider_info(&self) -> ProviderInfo {
        ProviderInfo {
            provider: self.provider,
            logo_url: match self.provider {
                Provider::Picnic => PicnicBridge::logo_url(),
                Provider::Jumbo => JumboBridge::logo_url(),
            },
        }
    }
}

#[derive(Serialize, Deserialize, async_graphql::Interface, Clone, Debug, PartialEq, PartialOrd)]
#[graphql(field(name = "id", type = "&String"))]
pub enum PromotionProduct {
    Product(WggSearchProduct),
    ProductId(ProductId),
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, Eq, PartialOrd)]
pub struct ProductId {
    pub id: String,
}
