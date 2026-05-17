//! Test program exercising account-wrapper types that sit outside the
//! constraints/seeds/cpi suites — Sysvar, Box<Account>, SystemAccount,
//! and bare UncheckedAccount read paths.

use {
    anchor_lang_v2::{
        prelude::*,
        programs::{AssociatedToken, Memo},
    },
    pinocchio::sysvars::{clock::Clock, rent::Rent},
};

declare_id!("Acc1111111111111111111111111111111111111111");

#[account]
pub struct Counter {
    pub value: u64,
}

#[program]
pub mod accounts_test {
    use super::*;

    /// Initialize a counter. Exercises `BorshAccount`'s init path through a
    /// regular `Account<T>` — also kicks the `Box<Account<T>>` handler below
    /// into a known state.
    #[discrim = 0]
    pub fn initialize(ctx: &mut Context<Initialize>) -> Result<()> {
        ctx.accounts.counter.value = 1;
        Ok(())
    }

    /// Loads the counter inside a `Box` and mutates it through `Deref`.
    /// Hits `AnchorAccount for Box<T>` (`accounts/boxed.rs`).
    #[discrim = 1]
    pub fn bump_boxed(ctx: &mut Context<BumpBoxed>) -> Result<()> {
        ctx.accounts.counter.value = ctx.accounts.counter.value.wrapping_add(1);
        Ok(())
    }

    /// Loads the counter immutably inside a `Box` and only reads through
    /// `Deref`. Hits `AnchorAccount::load` for `Box<T>`.
    #[discrim = 2]
    pub fn read_boxed(ctx: &mut Context<ReadBoxed>) -> Result<()> {
        let _ = ctx.accounts.counter.value;
        Ok(())
    }

    /// Initializes a boxed counter via `AccountInitialize for Box<T>`.
    #[discrim = 3]
    pub fn initialize_boxed(ctx: &mut Context<InitializeBoxed>) -> Result<()> {
        ctx.accounts.counter.value = 7;
        Ok(())
    }

    /// Closes a boxed counter, forwarding through `AnchorAccount::close`.
    #[discrim = 4]
    pub fn close_boxed(_ctx: &mut Context<CloseBoxed>) -> Result<()> {
        Ok(())
    }

    /// Reads the Clock sysvar. Exercises `Sysvar<Clock>::load` and `Deref`
    /// forwarding to the inner pinocchio type.
    #[discrim = 5]
    pub fn read_clock(ctx: &mut Context<ReadClock>) -> Result<()> {
        // Touch several Clock fields so the register trace covers the
        // deref/getter path.
        let clock = &*ctx.accounts.clock;
        let _ = clock.slot;
        let _ = clock.epoch;
        let _ = clock.unix_timestamp;
        Ok(())
    }

    /// Reads the Rent sysvar. Same rationale as `read_clock`.
    #[discrim = 6]
    pub fn read_rent(ctx: &mut Context<ReadRent>) -> Result<()> {
        let rent = &*ctx.accounts.rent;
        let _ = rent.try_minimum_balance(100);
        Ok(())
    }

    /// Take a `SystemAccount` — validates the passed account is owned by
    /// the System program. Exercises `accounts/system_account.rs`.
    #[discrim = 7]
    pub fn check_system(ctx: &mut Context<CheckSystem>) -> Result<()> {
        let _ = ctx.accounts.wallet.address();
        Ok(())
    }

    /// Read-only UncheckedAccount — exercises load + accessor paths on
    /// `accounts/unchecked_account.rs`.
    #[discrim = 8]
    pub fn touch_unchecked(ctx: &mut Context<TouchUnchecked>) -> Result<()> {
        let _ = ctx.accounts.any_account.address();
        Ok(())
    }

    /// Checks the well-known System program marker address.
    #[discrim = 9]
    pub fn check_system_program(ctx: &mut Context<CheckSystemProgram>) -> Result<()> {
        let _ = ctx.accounts.program.address();
        Ok(())
    }

    /// Checks the well-known SPL Token program marker address.
    #[discrim = 10]
    pub fn check_token_program(ctx: &mut Context<CheckTokenProgram>) -> Result<()> {
        let _ = ctx.accounts.program.address();
        Ok(())
    }

    /// Checks the well-known Token-2022 program marker address.
    #[discrim = 11]
    pub fn check_token_2022_program(ctx: &mut Context<CheckToken2022Program>) -> Result<()> {
        let _ = ctx.accounts.program.address();
        Ok(())
    }

    /// Checks the well-known Associated Token program marker address.
    #[discrim = 12]
    pub fn check_associated_token_program(
        ctx: &mut Context<CheckAssociatedTokenProgram>,
    ) -> Result<()> {
        let _ = ctx.accounts.program.address();
        Ok(())
    }

    /// Checks the well-known Memo program marker address.
    #[discrim = 13]
    pub fn check_memo_program(ctx: &mut Context<CheckMemoProgram>) -> Result<()> {
        let _ = ctx.accounts.program.address();
        Ok(())
    }
}

// -- Accounts structs --------------------------------------------------------

#[derive(Accounts)]
pub struct Initialize {
    #[account(mut)]
    pub payer: Signer,
    #[account(init, payer = payer, seeds = [b"counter"], bump)]
    pub counter: Account<Counter>,
    pub system_program: Program<System>,
}

#[derive(Accounts)]
pub struct BumpBoxed {
    #[account(mut)]
    pub counter: Box<Account<Counter>>,
}

#[derive(Accounts)]
pub struct ReadBoxed {
    #[account(seeds = [b"boxed-counter"], bump)]
    pub counter: Box<Account<Counter>>,
}

#[derive(Accounts)]
pub struct InitializeBoxed {
    #[account(mut)]
    pub payer: Signer,
    #[account(init, payer = payer, seeds = [b"boxed-counter"], bump)]
    pub counter: Box<Account<Counter>>,
    pub system_program: Program<System>,
}

#[derive(Accounts)]
pub struct CloseBoxed {
    #[account(mut, close = receiver, seeds = [b"boxed-counter"], bump)]
    pub counter: Box<Account<Counter>>,
    #[account(mut)]
    pub receiver: SystemAccount,
}

#[derive(Accounts)]
pub struct ReadClock {
    pub clock: Sysvar<Clock>,
}

#[derive(Accounts)]
pub struct ReadRent {
    pub rent: Sysvar<Rent>,
}

#[derive(Accounts)]
pub struct CheckSystem {
    pub wallet: SystemAccount,
}

#[derive(Accounts)]
pub struct TouchUnchecked {
    pub any_account: UncheckedAccount,
}

#[derive(Accounts)]
pub struct CheckSystemProgram {
    pub program: Program<System>,
}

#[derive(Accounts)]
pub struct CheckTokenProgram {
    pub program: Program<Token>,
}

#[derive(Accounts)]
pub struct CheckToken2022Program {
    pub program: Program<Token2022>,
}

#[derive(Accounts)]
pub struct CheckAssociatedTokenProgram {
    pub program: Program<AssociatedToken>,
}

#[derive(Accounts)]
pub struct CheckMemoProgram {
    pub program: Program<Memo>,
}

#[derive(Accounts)]
pub struct CheckAssociatedTokenProgramSeed {
    #[account(seeds = [b"vault"], bump, seeds::program = AssociatedToken::id())]
    pub data: UncheckedAccount,
}
