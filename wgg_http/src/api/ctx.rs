use crate::api::State;
use async_graphql::Context;

/// Retrieve the [`State`] from the context
#[inline]
pub(crate) fn get_state_from_ctx<'a>(ctx: &Context<'a>) -> &'a State {
    ctx.data_unchecked()
}
