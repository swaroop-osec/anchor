//! `dup-realloc` — reproducer for v2 **bug #3**: a `Realloc2`-shaped
//! accounts struct with two `unsafe(dup) + realloc` fields, invoked with
//! the same PDA in both slots, fails at runtime with `AccountBorrowFailed`.
//!
//! ```text
//! InstructionError(0, AccountBorrowFailed):
//!   "instruction tries to borrow reference for an account which is
//!    already borrowed"
//! ```
//!
//! The integration test (`tests-v2/tests/dup_realloc.rs`) drives this
//! through the standard `tests_v2::build_program` + LiteSVM pipeline.
//! The program **builds fine**; the test panics on the runtime error
//! when `realloc_aliased` is called with the same PDA in both slots.
//! That panic is the bug repro.

extern crate alloc;

use {alloc::vec::Vec, anchor_lang_v2::prelude::*};

declare_id!("DupRea11oc1111111111111111111111111111111111");

#[program]
pub mod dup_realloc {
    use super::*;

    #[discrim = 0]
    pub fn init(ctx: &mut Context<Init>) -> Result<()> {
        ctx.accounts.sample.data = alloc::vec![0];
        ctx.accounts.sample.bump = ctx.bumps.sample;
        Ok(())
    }

    /// Two `unsafe(dup)` fields with `realloc`. When invoked with the
    /// same PDA in both slots, v2's runtime rejects the second realloc
    /// with `AccountBorrowFailed` — even though the body never deref's
    /// both at once.
    ///
    /// In v1 (with the `dup` constraint), the equivalent handler grew
    /// the underlying account to the larger of the two new sizes
    /// (`len + 10`).
    #[discrim = 1]
    pub fn realloc_aliased(ctx: &mut Context<ReallocAliased>, len: u16) -> Result<()> {
        ctx.accounts
            .sample1
            .data
            .resize_with(len as usize, Default::default);
        ctx.accounts
            .sample2
            .data
            .resize_with((len + 10) as usize, Default::default);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Init {
    #[account(mut)]
    pub authority: Signer,
    #[account(
        init,
        payer = authority,
        // disc(8) + Vec len-prefix(4) + 1 byte data + bump(1) = 14
        space = 8 + 4 + 1 + 1,
        seeds = [b"sample"],
        bump,
    )]
    pub sample: BorshAccount<Sample>,
    pub system_program: Program<System>,
}

#[derive(Accounts)]
#[instruction(len: u16)]
pub struct ReallocAliased {
    #[account(mut)]
    pub authority: Signer,

    #[account(
        mut,
        seeds = [b"sample"],
        bump = sample1.bump,
        realloc = 8 + 4 + len as usize + 1,
        realloc_payer = authority,
        realloc_zero = false,
        unsafe(dup),
    )]
    pub sample1: BorshAccount<Sample>,

    #[account(
        mut,
        seeds = [b"sample"],
        bump = sample2.bump,
        realloc = 8 + 4 + (len as usize + 10) + 1,
        realloc_payer = authority,
        realloc_zero = false,
        unsafe(dup),
    )]
    pub sample2: BorshAccount<Sample>,

    pub system_program: Program<System>,
}

#[account(borsh)]
pub struct Sample {
    pub data: Vec<u8>,
    pub bump: u8,
}
