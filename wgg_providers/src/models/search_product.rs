use crate::models::{CentPrice, Provider, ProviderInfo, UnitPrice, UnitQuantity, WggDecorator, WggProduct};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
#[graphql(complex)]
pub struct WggSearchProduct {
    pub id: String,
    pub name: String,
    /// The full price of an article, ignoring any sales
    pub full_price: CentPrice,
    /// The present display price (taking into account active sales).
    pub display_price: CentPrice,
    /// The amount of weight/liters/pieces this product represents.
    pub unit_quantity: UnitQuantity,
    pub unit_price: Option<UnitPrice>,
    /// A small check to see if the current item is unavailable.
    ///
    /// `decorators` might contain more information as to the nature of the disruption.
    pub available: bool,
    /// Direct URL to product image.
    pub image_url: Option<String>,
    pub decorators: Vec<WggDecorator>,
    #[graphql(skip)]
    /// The grocery store which provided this item.
    pub provider: Provider,
}

#[async_graphql::ComplexObject]
impl WggSearchProduct {
    /// Grocery store information associated with this item
    async fn provider_info(&self) -> ProviderInfo {
        self.provider.as_provider_info()
    }
}

impl From<WggProduct> for WggSearchProduct {
    fn from(product: WggProduct) -> Self {
        WggSearchProduct {
            id: product.id,
            name: product.name,
            full_price: product.price_info.original_price,
            display_price: product.price_info.display_price,
            unit_quantity: product.unit_quantity,
            unit_price: product.price_info.unit_price,
            available: product.available,
            image_url: product.image_urls.into_iter().next(),
            decorators: product.decorators,
            provider: product.provider,
        }
    }
}
