use anchor_lang::prelude::*;

declare_id!("f88R86sonBNc2RERkitEyypBXf7h2r3fDrvvXpb7n7r");

#[program]
pub mod constraints_trait {
    use super::*;

    pub fn noop(ctx: Context<Noop>) -> Result<()> {
        Ok(())
    }

    pub fn init_counter(ctx: Context<InitCounter>, start: u64) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        counter.count = start;
        counter.authority = ctx.accounts.authority.key();
        // Persist PDA bump for later validation.
        counter.bump = ctx.bumps.counter;
        Ok(())
    }

    pub fn increment(ctx: Context<IncrementCounter>) -> Result<()> {
        // Ensure bump derivation is consistent across instructions.
        require_eq!(
            ctx.bumps.counter,
            ctx.accounts.counter.bump,
            ErrorCode::ConstraintSeeds
        );
        let counter = &mut ctx.accounts.counter;
        counter.count = counter.count.saturating_add(1);
        Ok(())
    }
}

#[derive(Accounts)]
#[accounts(manual_constraints)]
pub struct Noop<'info> {
    pub authority: Signer<'info>,
    pub system_program: AccountInfo<'info>,
    pub rent: AccountInfo<'info>,
    pub self_program: AccountInfo<'info>,
}

impl<'info> Constraints for Noop<'info> {
    fn validate<'ctx>(&self, ctx: &Context<'_, '_, '_, 'ctx, Self>) -> Result<()> {
        require!(self.authority.is_signer, ErrorCode::ConstraintSigner);
        require_keys_eq!(
            self.system_program.key(),
            system_program::ID,
            ErrorCode::ConstraintAddress
        );
        require!(
            self.system_program.executable,
            ErrorCode::ConstraintExecutable
        );
        require_keys_eq!(
            self.self_program.key(),
            *ctx.program_id,
            ErrorCode::ConstraintAddress
        );
        require!(
            self.self_program.executable,
            ErrorCode::ConstraintExecutable
        );
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitCounter<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Counter::INIT_SPACE,
        seeds = [b"counter", authority.key().as_ref()],
        bump
    )]
    pub counter: Account<'info, Counter>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct IncrementCounter<'info> {
    #[account(
        mut,
        seeds = [b"counter", authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub counter: Account<'info, Counter>,
    pub authority: Signer<'info>,
}

#[account]
pub struct Counter {
    pub count: u64,
    pub bump: u8,
    pub authority: Pubkey,
}

impl Counter {
    pub const INIT_SPACE: usize = 8 + 1 + 32;
}
