use crate::api::State;
use async_graphql::Context;

pub(crate) trait ContextExt {
    /// Retrieve the [`State`] from the context
    fn wgg_state(&self) -> &State;
}

impl<'a> ContextExt for Context<'a> {
    fn wgg_state(&self) -> &State {
        self.data_unchecked()
    }
}

/// Retrieve the [`State`] from the context
#[inline]
pub(crate) fn get_state_from_ctx<'a>(ctx: &Context<'a>) -> &'a State {
    ctx.data_unchecked()
}
