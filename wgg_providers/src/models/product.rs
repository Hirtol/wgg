use crate::models::{
    AllergyTags, IngredientInfo, ItemInfo, NutritionalInfo, PriceInfo, Provider, ProviderInfo, SaleInformation,
    TextType, UnitQuantity, WggDecorator,
};
use serde::{Deserialize, Serialize};

// ** Full Product **
#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
#[graphql(complex)]
pub struct WggProduct {
    /// This service's ID for the current product.
    /// Not transferable between [Provider]s
    pub id: String,
    /// The name of the product.
    pub name: String,
    /// Full product description.
    pub description: Description,
    /// All price related information
    pub price_info: PriceInfo,
    /// The amount of weight/liters/pieces this product represents.
    pub unit_quantity: UnitQuantity,
    /// A small check to see if the current item is unavailable.
    ///
    /// `decorators` might contains more information as to the nature of the disruption.
    pub available: bool,
    /// Direct URL to product image.
    pub image_urls: Vec<String>,
    /// All ingredients in a structured format.
    ///
    /// Can be empty for base ingredients such as cucumbers, for example.
    pub ingredients: Vec<IngredientInfo>,
    /// Denotes the nutritional info, normalised to 100g.
    pub nutritional: Option<NutritionalInfo>,
    /// All information for allergy tags.
    ///
    /// Can be empty if the product has no allergens.
    pub allergy_info: Vec<AllergyTags>,
    /// Denotes all optional bits of information, such as preparation instructions or supplier information.
    ///
    /// These can be useful to add as additional collapsable tabs in the front-end ui.
    pub additional_items: Vec<ItemInfo>,
    /// All decorators describing the object in further detail.
    pub decorators: Vec<WggDecorator>,
    /// Any information about sales relevant for this product.
    pub sale_information: Option<SaleInformation>,
    /// The grocery store this item is provided from.
    #[graphql(skip)]
    pub provider: Provider,
}

#[async_graphql::ComplexObject]
impl WggProduct {
    /// Grocery store information associated with this item
    async fn provider_info(&self) -> ProviderInfo {
        self.provider.as_provider_info()
    }
}

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, Eq, PartialOrd)]
pub struct Description {
    pub text: String,
    pub text_type: TextType,
}
