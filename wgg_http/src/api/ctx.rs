use async_graphql::Context;
use crate::api::State;

/// Retrive the [`State`] from the context
#[inline]
pub(crate) fn get_state_from_ctx<'a>(ctx: &Context<'a>) -> &'a State {
    ctx.data_unchecked()
}