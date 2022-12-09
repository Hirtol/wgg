use crate::models::{
    PriceInfo, Provider, ProviderInfo, SaleInformation, UnavailableItem, UnitQuantity, WggDecorator, WggProduct,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
#[graphql(complex)]
pub struct WggSearchProduct {
    /// This service's ID for the current product.
    /// Not transferable between [Provider]s
    pub id: String,
    /// The name of the product.
    pub name: String,
    /// All pricing info related to this object.
    pub price_info: PriceInfo,
    /// The amount of weight/liters/pieces this product represents.
    pub unit_quantity: UnitQuantity,
    /// If this product is currently unavailable this will contain details explaining why.
    ///
    /// If this is `None` then the object is available
    pub unavailable_details: Option<UnavailableItem>,
    /// Direct URL to product image.
    pub image_url: Option<String>,
    pub decorators: Vec<WggDecorator>,
    /// Any information about sales relevant for this product.
    pub sale_information: Option<SaleInformation>,
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
            price_info: product.price_info,
            unit_quantity: product.unit_quantity,
            unavailable_details: product.unavailable_details,
            image_url: product.image_urls.into_iter().next(),
            decorators: product.decorators,
            sale_information: product.sale_information,
            provider: product.provider,
        }
    }
}
