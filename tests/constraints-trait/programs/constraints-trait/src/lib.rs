use anchor_lang::prelude::*;

declare_id!("f88R86sonBNc2RERkitEyypBXf7h2r3fDrvvXpb7n7r");

#[program]
pub mod constraints_trait {
    use super::*;

    pub fn noop(ctx: Context<Noop>) -> Result<()> {
        ctx.validate()?;
        Ok(())
    }

    pub fn init_counter(ctx: Context<InitCounter>, start: u64) -> Result<()> {
        ctx.validate()?;
        let counter = &mut ctx.accounts.counter;
        counter.count = start;
        counter.authority = ctx.accounts.authority.key();
        Ok(())
    }

    pub fn increment(ctx: Context<IncrementCounter>) -> Result<()> {
        ctx.validate()?;
        let counter = &mut ctx.accounts.counter;
        counter.count = counter.count.saturating_add(1);
        Ok(())
    }
}

#[derive(Accounts)]
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
    #[account(init, payer = authority, space = 8 + Counter::INIT_SPACE)]
    pub counter: Account<'info, Counter>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Constraints for InitCounter<'info> {
    fn validate<'ctx>(&self, ctx: &Context<'_, '_, '_, 'ctx, Self>) -> Result<()> {
        require!(self.authority.is_signer, ErrorCode::ConstraintSigner);
        require!(self.authority.is_writable, ErrorCode::ConstraintMut);
        require!(
            self.counter.to_account_info().is_writable,
            ErrorCode::ConstraintMut
        );
        require_keys_eq!(
            self.system_program.key(),
            system_program::ID,
            ErrorCode::ConstraintAddress
        );
        require!(
            self.system_program.executable,
            ErrorCode::ConstraintExecutable
        );
        require!(
            self.counter.to_account_info().owner == ctx.program_id,
            ErrorCode::ConstraintOwner
        );
        let rent = Rent::get()?;
        require!(
            rent.is_exempt(
                self.counter.to_account_info().lamports(),
                self.counter.to_account_info().data_len()
            ),
            ErrorCode::ConstraintRentExempt
        );
        Ok(())
    }
}

#[derive(Accounts)]
pub struct IncrementCounter<'info> {
    #[account(mut)]
    pub counter: Account<'info, Counter>,
    pub authority: Signer<'info>,
}

impl<'info> Constraints for IncrementCounter<'info> {
    fn validate<'ctx>(&self, ctx: &Context<'_, '_, '_, 'ctx, Self>) -> Result<()> {
        require!(
            self.counter.to_account_info().is_writable,
            ErrorCode::ConstraintMut
        );
        require!(self.authority.is_signer, ErrorCode::ConstraintSigner);
        require_keys_eq!(
            self.counter.authority,
            self.authority.key(),
            ErrorCode::ConstraintHasOne
        );
        require!(
            self.counter.to_account_info().owner == ctx.program_id,
            ErrorCode::ConstraintOwner
        );
        Ok(())
    }
}

#[account]
pub struct Counter {
    pub count: u64,
    pub authority: Pubkey,
}

impl Counter {
    pub const INIT_SPACE: usize = 8 + 32;
}
