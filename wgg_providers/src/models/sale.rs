use crate::models::sale_types::SaleType;
use crate::models::{Provider, ProviderInfo, SaleValidity, SublistId, WggSearchProduct};
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
    /// Contains all info related to the sale
    pub sale_info: SaleInformation,
    /// The description of this particular sale group.
    pub sale_description: Option<String>,
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
    /// Contains all info related to the sale
    pub sale_info: SaleInformation,
    /// The description of this particular sale group.
    pub sale_description: Option<String>,
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

#[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
pub struct SaleInformation {
    /// A label for a sale like `1 + 1 GRATIS` or `2 voor 2.50`.
    /// Used to derived [Self::sale_type]
    pub label: String,
    /// Additional labels relevant for this particular item such as:
    /// * `Only online`.
    pub additional_label: Vec<String>,
    /// From and to when this sale is valid.
    ///
    /// Some providers may make a best guess so this information isn't always 100% accurate.
    pub sale_validity: SaleValidity,
    /// The derived sale type used for cart analysis.
    /// If this couldn't be derived this will be `None`, and should indicate to the user some caution as any relevant
    /// sale is not taken into account.
    pub sale_type: Option<SaleType>,
}

pub mod sale_types {
    use crate::models::CentPrice;
    use serde::{Deserialize, Serialize};
    use std::num::NonZeroU16;

    #[derive(Serialize, Deserialize, async_graphql::Union, Clone, Debug, PartialEq, PartialOrd)]
    pub enum SaleType {
        NumPlusNumFree(NumPlusNumFree),
        NumPercentOff(NumPercentOff),
        NumthPercentOff(NumthPercentOff),
        NumForPrice(NumForPrice),
        NumEuroOff(NumEuroOff),
        NumEuroPrice(NumEuroPrice),
    }

    /// Follows from the following kinds of sales:
    /// * `1 + 1 GRATIS` - 1 required and 1 free
    /// * `4 + 2 GRATIS` - 4 required and 2 free
    /// * `2e GRATIS` - 1 required and 1 free
    #[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
    pub struct NumPlusNumFree {
        pub required: NonZeroU16,
        pub free: NonZeroU16,
    }

    /// Follows from the following kinds of sales:
    /// * `20% OFF`
    /// * `50% OFF`
    #[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
    pub struct NumPercentOff {
        /// The percent reduction. Guaranteed to be `> 0 && <= 100`.
        percent_off: NonZeroU16,
    }

    impl NumPercentOff {
        pub fn new(percent_off: impl TryInto<NonZeroU16>) -> Option<Self> {
            let percent_off = percent_off.try_into().ok()?;
            if percent_off.get() > 100 {
                None
            } else {
                Self { percent_off }.into()
            }
        }

        /// Get the percent off.
        ///
        /// Guaranteed to be `> 0 && <= 100`.
        pub fn get_percent_off(&self) -> NonZeroU16 {
            self.percent_off
        }
    }

    /// Follows from the following kinds of sales:
    /// * `2e HALVE PRIJS` - 2 required, last 50% off
    #[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
    pub struct NumthPercentOff {
        pub required: NonZeroU16,
        pub last_percent_off: NonZeroU16,
    }

    /// Follows from the following kinds of sales:
    /// * `3 voor €4,50` - 3 required, 450 centprice
    /// * `4 voor €2,50` - 4 required, 250 centprice
    #[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
    pub struct NumForPrice {
        pub required: NonZeroU16,
        pub price: CentPrice,
    }

    /// Follows from the following kinds of sales:
    /// * `1 EURO KORTING` - 100 centprice off
    #[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
    pub struct NumEuroOff {
        pub price_off: CentPrice,
    }

    /// Follows from the following kinds of sales:
    /// * `NU  €4.00` - 400 centprice
    #[derive(Serialize, Deserialize, async_graphql::SimpleObject, Clone, Debug, PartialEq, PartialOrd)]
    pub struct NumEuroPrice {
        pub price: CentPrice,
    }
}
