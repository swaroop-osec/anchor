use anchor_lang_v2::__alloc::{string::String, vec::Vec};
use anchor_lang_v2::prelude::*;

declare_id!("BF748KR4UhPq7xbhFQYd7yFKmh5UYdqed9GbD6oZvEyu");

pub const RETURN_SEED: &[u8] = b"return-store";

#[account]
#[repr(C)]
pub struct ReturnStore {
    pub authority: Address,
    pub last_base: u64,
    pub last_result: u64,
    pub calls: u16,
    pub bump: u8,
    pub _padding: [u8; 5],
}

#[derive(Clone, wincode::SchemaRead, wincode::SchemaWrite)]
pub struct ReturnPayload {
    pub amount: u64,
    pub label: String,
    pub samples: Vec<u16>,
    pub maybe_authority: Option<Address>,
}

#[program]
pub mod return_callee {
    use super::*;

    #[discrim = 0]
    pub fn initialize(ctx: &mut Context<Initialize>) -> Result<()> {
        ctx.accounts.data.authority = *ctx.accounts.authority.address();
        ctx.accounts.data.last_base = 0;
        ctx.accounts.data.last_result = 0;
        ctx.accounts.data.calls = 0;
        ctx.accounts.data.bump = ctx.bumps.data;
        ctx.accounts.data._padding = [0; 5];
        Ok(())
    }

    #[discrim = 1]
    pub fn calculate(ctx: &mut Context<Calculate>, base: u64, bonus: u16) -> Result<u64> {
        require_keys_eq!(
            ctx.accounts.data.authority,
            *ctx.accounts.authority.address()
        );
        ctx.accounts.data.calls = ctx.accounts.data.calls.saturating_add(1);
        let result = base
            .saturating_mul(2)
            .saturating_add(bonus as u64)
            .saturating_add(ctx.accounts.data.calls as u64);
        ctx.accounts.data.last_base = base;
        ctx.accounts.data.last_result = result;
        Ok(result)
    }

    #[discrim = 2]
    pub fn describe(ctx: &mut Context<Calculate>, base: u64, bonus: u16) -> Result<ReturnPayload> {
        require_keys_eq!(
            ctx.accounts.data.authority,
            *ctx.accounts.authority.address()
        );
        ctx.accounts.data.calls = ctx.accounts.data.calls.saturating_add(1);
        let amount = base
            .saturating_mul(3)
            .saturating_add(bonus as u64)
            .saturating_add(ctx.accounts.data.calls as u64);
        ctx.accounts.data.last_base = base;
        ctx.accounts.data.last_result = amount;
        let mut samples = Vec::new();
        samples.push(bonus);
        samples.push(ctx.accounts.data.calls);
        Ok(ReturnPayload {
            amount,
            label: String::from("return-payload"),
            samples,
            maybe_authority: Some(*ctx.accounts.authority.address()),
        })
    }
}

#[derive(Accounts)]
pub struct Initialize {
    #[account(mut)]
    pub payer: Signer,
    #[account(
        init,
        payer = payer,
        space = 8 + core::mem::size_of::<ReturnStore>(),
        seeds = [RETURN_SEED, authority.address().as_ref()],
        bump,
    )]
    pub data: Account<ReturnStore>,
    pub authority: Signer,
    pub system_program: Program<System>,
}

#[derive(Accounts)]
pub struct Calculate {
    #[account(mut, seeds = [RETURN_SEED, authority.address().as_ref()], bump)]
    pub data: Account<ReturnStore>,
    pub authority: Signer,
}
