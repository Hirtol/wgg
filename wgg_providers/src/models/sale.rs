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
    /// May contain a image for a 'More' button
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
        self.provider.as_provider_info()
    }
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
#[graphql(complex)]
pub struct WggSaleCategoryComplete {
    pub id: String,
    pub name: String,
    pub image_urls: Vec<String>,
    /// All items that are part of this promotion.
    pub items: Vec<WggSearchProduct>,
    pub decorators: Vec<WggDecorator>,
    #[graphql(skip)]
    pub provider: Provider,
}

#[async_graphql::ComplexObject]
impl WggSaleCategoryComplete {
    /// Grocery store information associated with this item
    async fn provider_info(&self) -> ProviderInfo {
        self.provider.as_provider_info()
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
