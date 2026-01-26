use crate::prelude::*;
use crate::Bumps;
pub trait Constraints: Bumps + Sized {
    fn validate<'info>(&self, ctx: &Context<'_, '_, '_, 'info, Self>) -> Result<()>;
}
