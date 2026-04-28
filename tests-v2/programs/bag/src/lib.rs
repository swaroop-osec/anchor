//! `bag` — a tiny `PodVec`-backed account that fires two v2 bugs at
//! once when compiled with the documented minimal feature set.
//!
//! Bugs reproduced:
//!
//!   - **#1** — the `require_*!` macro chain leaks `solana_program_log`
//!     to the user crate. The `try_add` handler uses `require_eq!` with
//!     a custom error code; expansion reaches `solana_program_log::log!`
//!     which emits absolute paths the user crate can't resolve.
//!     Fails to compile with `error[E0433]: failed to resolve: could
//!     not find solana_program_log in the list of imported crates`.
//!
//!   - **#19** — `CapacityError` is not in the prelude and lang-v2
//!     ships no `From<CapacityError> for ProgramError`. Calling
//!     `try_push(x)?` is the natural form, but `?` can't convert the
//!     error. Fails to compile with `error[E0277]: \`?\` couldn't
//!     convert the error to ProgramError`.
//!
//! Both errors fire from the same `try_add` handler. The integration
//! test in `tests-v2/tests/bag.rs` follows the standard
//! `tests_v2::build_program` + LiteSVM pipeline; the SBF compile fails
//! today, `setup()` panics, every test panics. That panic is the bug
//! repro — the day v2 fixes either or both, the program builds and the
//! tests run to completion.

extern crate alloc;

use anchor_lang_v2::prelude::*;

declare_id!("BagBugBag11111111111111111111111111111111111");

const MAX_ITEMS: usize = 8;

#[account]
#[repr(C)]
pub struct Bag {
    pub items: PodVec<u64, MAX_ITEMS>,
}

#[program]
pub mod bag {
    use super::*;

    #[discrim = 0]
    pub fn init(_ctx: &mut Context<Init>) -> Result<()> {
        Ok(())
    }

    /// The natural way to write a "validated push": check the current
    /// length, then push. Both lines fire a different v2 bug.
    #[discrim = 1]
    pub fn try_add(ctx: &mut Context<Mut>, value: u64, expected_len: u64) -> Result<()> {
        // Bug #1: `require_eq!` with a custom error and format args
        // expands to `$crate::msg!` → `$crate::__log_impl!` →
        // `solana_program_log::log!`, which emits absolute
        // `::solana_program_log::*` paths at this call site. The user
        // crate doesn't depend on `solana-program-log`, so resolution
        // fails with E0433.
        require_eq!(
            ctx.accounts.bag.items.len() as u64,
            expected_len,
            BagError::WrongLength
        );

        // Bug #19: `try_push` returns `Result<(), CapacityError>`. The
        // `?` operator wants a `From<CapacityError> for ProgramError`
        // impl. lang-v2 ships none. Compile fails with E0277.
        ctx.accounts.bag.items.try_push(value)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Init {
    #[account(mut)]
    pub payer: Signer,
    #[account(init, payer = payer, seeds = [b"bag"], bump)]
    pub bag: Account<Bag>,
    pub system_program: Program<System>,
}

#[derive(Accounts)]
pub struct Mut {
    #[account(mut, seeds = [b"bag"], bump)]
    pub bag: Account<Bag>,
}

#[error_code]
pub enum BagError {
    #[msg("bag length did not match the expected value")]
    WrongLength,
}
