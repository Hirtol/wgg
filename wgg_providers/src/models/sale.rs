use crate::models::{Provider, ProviderInfo, SaleValidity, SublistId, WggDecorator, WggSearchProduct};
use serde::{Deserialize, Serialize};

// ** Promotions **
#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
#[graphql(complex)]
pub struct WggSaleCategory {
    /// If this category has an ID then it usually means more items can be requested for display in the `sublist` function.
    pub id: Option<SublistId>,
    /// The title of this category, which can contain multiple sale groups.
    /// Category names like 'Aardappel, rijst, pasta', or 'Groente, Fruit' are common.
    pub name: String,
    /// All groups/products relevant for this category. For certain (Picnic) APIs this will be just `Product` instances.
    /// For others like Jumbo, this might be just `Group`s. For still other's (AH) this can be a mix of both.
    pub items: Vec<WggSaleItem>,
    /// May contain a image for a 'More' button
    pub image_urls: Vec<String>,
    /// Indicates whether to display a 'More' button for this category or not.
    ///
    /// If `true` one can request more information by calling the `sublist` function with this category's ID.
    pub complete: bool,
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

#[derive(Serialize, Deserialize, async_graphql::Interface, Clone, Debug, PartialEq, PartialOrd)]
#[graphql(field(name = "id", type = "&String"))]
pub enum WggSaleItem {
    Product(WggSearchProduct),
    Group(WggSaleGroupLimited),
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
#[graphql(complex)]
pub struct WggSaleGroupLimited {
    pub id: SublistId,
    pub name: String,
    /// May contain a image for a 'More' button
    pub image_urls: Vec<String>,
    /// A list of only product Ids.
    pub items: Vec<ProductIdT>,
    /// Until when this sale is valid.
    ///
    /// Note that some providers may make a best guess for this if the original API does not provide it.
    pub sale_validity: SaleValidity,
    pub decorators: Vec<WggDecorator>,
    #[graphql(skip)]
    pub provider: Provider,
}

#[async_graphql::ComplexObject]
impl WggSaleGroupLimited {
    /// Grocery store information associated with this item
    async fn provider_info(&self) -> ProviderInfo {
        self.provider.as_provider_info()
    }
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
#[graphql(complex)]
pub struct WggSaleGroupComplete {
    pub id: SublistId,
    pub name: String,
    pub image_urls: Vec<String>,
    /// All items that are part of this promotion.
    pub items: Vec<WggSearchProduct>,
    pub decorators: Vec<WggDecorator>,
    /// Until when this sale is valid.
    ///
    /// Note that some providers may make a best guess for this if the original API does not provide it.
    pub sale_validity: SaleValidity,
    #[graphql(skip)]
    pub provider: Provider,
}

#[async_graphql::ComplexObject]
impl WggSaleGroupComplete {
    /// Grocery store information associated with this item
    async fn provider_info(&self) -> ProviderInfo {
        self.provider.as_provider_info()
    }
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, Eq, PartialOrd)]
pub struct ProductIdT {
    pub id: String,
}
