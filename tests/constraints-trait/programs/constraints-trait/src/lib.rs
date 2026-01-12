use anchor_lang::prelude::*;

declare_id!("f88R86sonBNc2RERkitEyypBXf7h2r3fDrvvXpb7n7r");

#[program]
pub mod constraints_trait {
    use super::*;

    pub fn noop(ctx: Context<Noop>) -> Result<()> {
        ctx.validate()?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Noop<'info> {
    pub authority: Signer<'info>,
}

impl<'info> Constraints for Noop<'info> {
    fn validate<'ctx>(&self, _ctx: &Context<'_, '_, '_, 'ctx, Self>) -> Result<()> {
        require!(self.authority.is_signer, ErrorCode::ConstraintSigner);
        Ok(())
    }
}
