use crate::api::{ContextExt, GraphqlResult};
use crate::db;
use crate::db::Id;
use async_graphql::Context;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::borrow::Cow;
use wgg_providers::models::{
    UnavailableItem, WggProduct, WggSaleCategory, WggSaleGroupComplete, WggSaleGroupLimited, WggSaleItem,
    WggSearchProduct,
};

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

// ** Implementations **

#[derive(Debug, async_graphql::SimpleObject, Clone)]
pub struct ProductCartInfo {
    /// Whether this product is part of the current cart or not
    pub is_part_of_current_cart: bool,
    /// The quantity of items selected, only present if `is_part_of_current_cart` is `true`.
    pub quantity: Option<u32>,
}

#[async_graphql::ComplexObject]
impl WggSearchProductWrapper {
    pub async fn id(&self) -> &String {
        &self.item.id
    }

    /// Return the cart info for the current viewer.
    pub async fn cart_info(&self, ctx: &Context<'_>) -> GraphqlResult<ProductCartInfo> {
        let state = ctx.wgg_state();
        let user = ctx.wgg_user()?;

        let cart_content = db::cart_contents::raw_product::Entity::find()
            .filter(db::cart_contents::raw_product::Column::ProviderProduct.eq(self.item.id.as_str()))
            .left_join(db::cart::Entity)
            .filter(db::cart::has_user(user.id))
            .one(&state.db)
            .await?;

        if let Some(content) = cart_content {
            Ok(ProductCartInfo {
                is_part_of_current_cart: true,
                quantity: Some(content.quantity as u32),
            })
        } else {
            Ok(ProductCartInfo {
                is_part_of_current_cart: false,
                quantity: None,
            })
        }
    }
}

#[async_graphql::ComplexObject]
impl WggProductWrapper {
    /// Return the cart info for the current viewer.
    pub async fn cart_info(&self, ctx: &Context<'_>) -> GraphqlResult<ProductCartInfo> {
        let state = ctx.wgg_state();
        let user = ctx.wgg_user()?;

        let cart_content = db::cart_contents::raw_product::Entity::find()
            .filter(db::cart_contents::raw_product::Column::ProviderProduct.eq(self.item.id.as_str()))
            .left_join(db::cart::Entity)
            .filter(db::cart::has_user(user.id))
            .one(&state.db)
            .await?;

        if let Some(content) = cart_content {
            Ok(ProductCartInfo {
                is_part_of_current_cart: true,
                quantity: Some(content.quantity as u32),
            })
        } else {
            Ok(ProductCartInfo {
                is_part_of_current_cart: false,
                quantity: None,
            })
        }
    }
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
