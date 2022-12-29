use crate::api::aggregate_ingredients::AggregateIngredient;
use crate::api::{ContextExt, GraphqlResult};
use crate::db::Id;
use async_graphql::Context;
use std::borrow::Cow;
use wgg_providers::models::{
    Provider, SublistId, UnavailableItem, WggProduct, WggSaleCategory, WggSaleGroupComplete, WggSaleGroupLimited,
    WggSaleItem, WggSearchProduct,
};

// ** Implementations **

#[derive(Debug, Clone)]
pub struct ProductAppInfo<'a> {
    pub product_id: &'a str,
    pub provider: Provider,
}

#[async_graphql::Object]
impl<'a> ProductAppInfo<'a> {
    /// Retrieve the direct quantity of this product within the given `cart_id`.
    ///
    /// If `cart_id` is not given then the current cart of the user is assumed.
    ///
    /// For indirect quantities please refer to [associated_aggregates].
    pub async fn direct_quantity(&self, ctx: &Context<'_>, cart_id: Option<Id>) -> GraphqlResult<Option<u32>> {
        let state = ctx.wgg_state();
        let user = ctx.wgg_user()?;
        let provider_id = state.provider_id_from_provider(&self.provider);
        crate::api::cart::get_direct_product_quantity(&state.db, cart_id, user.id, provider_id, self.product_id).await
    }

    /// Retrieve all associated [AggregateIngredient]s for this given product.
    pub async fn associated_aggregates(&self, ctx: &Context<'_>) -> GraphqlResult<Vec<AggregateIngredient>> {
        let state = ctx.wgg_state();
        let user = ctx.wgg_user()?;
        let provider_id = state.provider_id_from_provider(&self.provider);
        crate::api::aggregate_ingredients::get_associated_aggregate_for_product(
            &state.db,
            user.id,
            provider_id,
            self.product_id,
        )
        .await
    }
}

#[async_graphql::ComplexObject]
impl WggSearchProductWrapper {
    pub async fn id(&self) -> &String {
        &self.item.id
    }

    /// Return `Wgg` specific information for this product
    pub async fn app_info(&self) -> ProductAppInfo<'_> {
        ProductAppInfo {
            product_id: &self.item.id,
            provider: self.item.provider,
        }
    }

    /// Return, if there is a sale, the sale's id
    #[tracing::instrument(skip(ctx))]
    pub async fn sale_id(&self, ctx: &Context<'_>) -> Option<SublistId> {
        let state = ctx.wgg_state();
        state.providers.product_sale_id(self.item.provider, &self.item.id)
    }
}

#[async_graphql::ComplexObject]
impl WggProductWrapper {
    /// Return `Wgg` specific information for this product
    pub async fn app_info(&self) -> ProductAppInfo<'_> {
        ProductAppInfo {
            product_id: &self.item.id,
            provider: self.item.provider,
        }
    }

    /// Return, if there is a sale, the sale's id
    #[tracing::instrument(skip(ctx))]
    pub async fn sale_id(&self, ctx: &Context<'_>) -> Option<SublistId> {
        let state = ctx.wgg_state();
        state.providers.product_sale_id(self.item.provider, &self.item.id)
    }
}

// ** Wrapper Objects **
/// If the item is unavailable
#[derive(async_graphql::SimpleObject, Debug, Clone)]
#[graphql(name_type)]
pub struct UnavailableItemWrapper {
    #[graphql(flatten)]
    pub item: UnavailableItem,
    /// Lists replacements if the store has suggested any.
    ///
    /// Some stores won't support this functionality, and this would therefore remain empty.
    pub replacements: Vec<WggSearchProductWrapper>,
}

#[derive(Debug, async_graphql::SimpleObject, Clone)]
#[graphql(complex, name_type)]
pub struct WggSearchProductWrapper {
    #[graphql(flatten)]
    pub item: WggSearchProduct,
    pub unavailable_details: Option<UnavailableItemWrapper>,
}

#[derive(Debug, async_graphql::SimpleObject, Clone)]
#[graphql(complex, name_type)]
pub struct WggProductWrapper {
    #[graphql(flatten)]
    pub item: WggProduct,
    pub unavailable_details: Option<UnavailableItemWrapper>,
}

#[derive(async_graphql::Interface, Clone, Debug)]
#[graphql(field(name = "id", type = "&String"), name = "WggSaleItem")]
pub enum WggSaleItemWrapper {
    Product(WggSearchProductWrapper),
    Group(WggSaleGroupLimited),
}

#[derive(Debug, async_graphql::SimpleObject, Clone)]
#[graphql(name_type)]
pub struct WggSaleCategoryWrapper {
    #[graphql(flatten)]
    pub item: WggSaleCategory,
    /// All groups/products relevant for this category. For certain (Picnic) APIs this will be just `Product` instances.
    /// For others like Jumbo, this might be just `Group`s. For still other's (AH) this can be a mix of both.
    pub items: Vec<WggSaleItemWrapper>,
}

#[derive(Debug, async_graphql::SimpleObject, Clone)]
#[graphql(name_type)]
pub struct WggSaleGroupCompleteWrapper {
    #[graphql(flatten)]
    pub item: WggSaleGroupComplete,
    /// All items that are part of this promotion.
    pub items: Vec<WggSearchProductWrapper>,
}

impl async_graphql::TypeName for UnavailableItemWrapper {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("UnavailableItem")
    }
}

impl From<UnavailableItem> for UnavailableItemWrapper {
    fn from(mut value: UnavailableItem) -> Self {
        let replacements = std::mem::take(&mut value.replacements);
        UnavailableItemWrapper {
            replacements: replacements.into_iter().map(|i| i.into()).collect(),
            item: value,
        }
    }
}

impl async_graphql::TypeName for WggProductWrapper {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("WggProduct")
    }
}

impl From<WggProduct> for WggProductWrapper {
    fn from(mut item: WggProduct) -> Self {
        Self {
            unavailable_details: item.unavailable_details.take().map(|i| i.into()),
            item,
        }
    }
}

impl async_graphql::TypeName for WggSearchProductWrapper {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("WggSearchProduct")
    }
}

impl From<WggSearchProduct> for WggSearchProductWrapper {
    fn from(mut item: WggSearchProduct) -> Self {
        Self {
            unavailable_details: item.unavailable_details.take().map(|i| i.into()),
            item,
        }
    }
}

impl From<WggSaleItem> for WggSaleItemWrapper {
    fn from(item: WggSaleItem) -> Self {
        match item {
            WggSaleItem::Product(prod) => Self::Product(prod.into()),
            WggSaleItem::Group(group) => Self::Group(group),
        }
    }
}

impl async_graphql::TypeName for WggSaleCategoryWrapper {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("WggSaleCategory")
    }
}

impl From<WggSaleCategory> for WggSaleCategoryWrapper {
    fn from(mut item: WggSaleCategory) -> Self {
        let replacements = std::mem::take(&mut item.items);
        Self {
            items: replacements.into_iter().map(|i| i.into()).collect(),
            item,
        }
    }
}

impl async_graphql::TypeName for WggSaleGroupCompleteWrapper {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("WggSaleGroupComplete")
    }
}

impl From<WggSaleGroupComplete> for WggSaleGroupCompleteWrapper {
    fn from(mut item: WggSaleGroupComplete) -> Self {
        let replacements = std::mem::take(&mut item.items);
        Self {
            items: replacements.into_iter().map(|i| i.into()).collect(),
            item,
        }
    }
}
