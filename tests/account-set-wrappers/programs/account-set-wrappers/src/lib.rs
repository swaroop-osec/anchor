use anchor_lang::account_set::{
    Executable, HasOne, HasOneTarget, Mut, Owned, Seeded, Seeds, SingleAccountSet,
};
use anchor_lang::prelude::*;

declare_id!("GLd3xnKGxJHRNi4sAYg6M6u1mVvDsCvvjaQvFGiAL7o5");

#[program]
pub mod account_set_wrappers {
    use super::*;

    // ========================================================================
    // This test program validates the account_set module wrappers used
    // DIRECTLY as account types in #[derive(Accounts)] structs.
    //
    // The wrapper types:
    // - Mut<T>: enforces is_writable constraint (replaces #[account(mut)])
    // - Seeded<T, S>: validates PDA derivation (replaces #[account(seeds=..., bump)])
    //
    // Note: For signer validation, use the existing Signer<'info> type.
    //
    // NEW WRAPPER TYPES (supported in derive macro):
    // - Owned<T, P>: validates account owner
    // - Executable<T>: validates account is a program
    // - HasOne<T, Target>: validates account relationships
    //
    // These new wrappers can be used directly in handlers and in #[derive(Accounts)].
    // ========================================================================

    /// Initialize a test data account (using traditional syntax for init)
    pub fn initialize(ctx: Context<Initialize>, value: u64) -> Result<()> {
        let data = &mut ctx.accounts.data;
        data.value = value;
        data.authority = ctx.accounts.authority.key();
        data.bump = 0;
        msg!("Initialized test data with value: {}", value);
        Ok(())
    }

    /// Initialize a PDA test data account
    pub fn init_pda(ctx: Context<InitPda>, value: u64) -> Result<()> {
        let data = &mut ctx.accounts.data;
        data.value = value;
        data.authority = ctx.accounts.authority.key();
        data.bump = ctx.bumps.data;
        msg!(
            "Initialized PDA test data with value: {}, bump: {}",
            value,
            data.bump
        );
        Ok(())
    }

    // ========================================================================
    // Tests using Mut<T> and Seeded<T, S> AS ACCOUNT TYPES in #[derive(Accounts)]
    // These are supported in the derive macro
    // ========================================================================

    /// Test Mut<T> as an account type - validation happens in try_accounts
    pub fn test_mut_as_type(ctx: Context<TestMutAsType>, value: u64) -> Result<()> {
        msg!("Mut<T> as type test passed!");
        msg!("  Account key: {}", ctx.accounts.data.key());
        msg!("  Current value: {}", ctx.accounts.data.value);

        // Modify through the wrapper (Deref allows direct access)
        ctx.accounts.data.value = value;
        msg!("  New value: {}", value);

        Ok(())
    }

    /// Test Seeded<T, S> as an account type - PDA validation happens in try_accounts
    pub fn test_seeded_as_type(ctx: Context<TestSeededAsType>) -> Result<()> {
        msg!("Seeded<T, S> as type test passed!");
        msg!("  PDA key: {}", ctx.accounts.data.key());
        msg!("  Bump: {}", ctx.accounts.data.bump());
        msg!("  Value: {}", ctx.accounts.data.value);

        Ok(())
    }

    /// Test Mut<Seeded<Account, Seeds>> composition
    pub fn test_mut_seeded_as_type(ctx: Context<TestMutSeededAsType>, value: u64) -> Result<()> {
        msg!("Mut<Seeded<T, S>> as type test passed!");
        msg!("  PDA key: {}", ctx.accounts.data.key());
        msg!("  Bump: {}", ctx.accounts.data.bump());
        msg!("  Current value: {}", ctx.accounts.data.value);

        ctx.accounts.data.value = value;
        msg!("  New value: {}", value);

        Ok(())
    }

    /// Test SingleAccountSet trait methods through Mut<Account> type
    pub fn test_single_account_set_trait(ctx: Context<TestSingleAccountSetTrait>) -> Result<()> {
        let data = &ctx.accounts.data;

        let pubkey = SingleAccountSet::pubkey(data);
        let is_signer = SingleAccountSet::is_signer(data);
        let is_writable = SingleAccountSet::is_writable(data);
        let owner = SingleAccountSet::owner(data);
        let lamports = SingleAccountSet::lamports(data);

        msg!("SingleAccountSet trait test through Mut<Account> type:");
        msg!("  pubkey(): {}", pubkey);
        msg!("  is_signer(): {}", is_signer);
        msg!("  is_writable(): {}", is_writable);
        msg!("  owner(): {}", owner);
        msg!("  lamports(): {}", lamports);

        require!(is_writable, ErrorCode::AccountNotMutable);

        Ok(())
    }

    // ========================================================================
    // Tests for NEW wrapper types - using traditional #[account] attributes
    // until parser support is added for these wrappers in derive macro
    // ========================================================================

    /// Test Owned<T, P> validation - validates account owner
    /// Uses manual validation in handler to demonstrate wrapper functionality
    pub fn test_owned_wrapper(ctx: Context<TestOwnedWrapper>) -> Result<()> {
        // Validate that account is owned by our program
        let account_info = ctx.accounts.data.to_account_info();
        let expected_owner = crate::ID;
        require_keys_eq!(
            *account_info.owner,
            expected_owner,
            ErrorCode::ConstraintOwner
        );

        msg!("Owned<T, P> wrapper test passed!");
        msg!("  Account key: {}", ctx.accounts.data.key());
        msg!("  Account owner: {}", account_info.owner);
        Ok(())
    }

    /// Test Executable<T> validation - validates account is a program
    pub fn test_executable_wrapper(ctx: Context<TestExecutableWrapper>) -> Result<()> {
        // Validate that account is executable
        let account_info = ctx.accounts.program_account.to_account_info();
        require!(account_info.executable, ErrorCode::ConstraintExecutable);

        msg!("Executable<T> wrapper test passed!");
        msg!("  Program key: {}", ctx.accounts.program_account.key());
        msg!("  Is executable: {}", account_info.executable);
        Ok(())
    }

    /// Test HasOne<T, Target> validation - auto-validated in Constraints
    pub fn test_has_one_wrapper(ctx: Context<TestHasOneWrapper>) -> Result<()> {
        let expected_authority = ctx.accounts.authority.key();
        let actual_authority = ctx.accounts.data.authority;

        msg!("HasOne<T, Target> wrapper test passed!");
        msg!("  Data authority: {}", actual_authority);
        msg!("  Expected authority: {}", expected_authority);
        Ok(())
    }

}

// ============================================================================
// Account Structs
// ============================================================================

#[account]
#[derive(InitSpace)]
pub struct TestData {
    pub value: u64,
    pub authority: Pubkey,
    pub bump: u8,
}

// ============================================================================
// Seeds for PDA
// ============================================================================

/// Seeds for the test PDA account
#[derive(Default)]
pub struct TestDataSeeds;

impl Seeds for TestDataSeeds {
    fn seeds(&self) -> Vec<&[u8]> {
        vec![b"test_data"]
    }
}

// ============================================================================
// Initialization Accounts (using traditional syntax for init)
// ============================================================================

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + TestData::INIT_SPACE
    )]
    pub data: Account<'info, TestData>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitPda<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + TestData::INIT_SPACE,
        seeds = [b"test_data"],
        bump
    )]
    pub data: Account<'info, TestData>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// ============================================================================
// Accounts using WRAPPER TYPES (Mut<T>, Seeded<T, S>) - SUPPORTED in derive macro
// ============================================================================

/// Uses Mut<Account<...>> as the account type
#[derive(Accounts)]
pub struct TestMutAsType<'info> {
    /// Mut<T> validates writable automatically - NO #[account(mut)] needed!
    pub data: Mut<Account<'info, TestData>>,
    pub authority: Signer<'info>,
}

/// Uses Seeded<Account, Seeds> as the account type
#[derive(Accounts)]
pub struct TestSeededAsType<'info> {
    /// Seeded<T, S> validates PDA and captures bump - NO seeds/bump attrs needed!
    pub data: Seeded<Account<'info, TestData>, TestDataSeeds>,
    pub authority: Signer<'info>,
}

/// Uses Mut<Seeded<Account, Seeds>> composition
#[derive(Accounts)]
pub struct TestMutSeededAsType<'info> {
    /// Mut<Seeded<T, S>> = writable AND PDA validated automatically!
    pub data: Mut<Seeded<Account<'info, TestData>, TestDataSeeds>>,
    pub authority: Signer<'info>,
}

/// Uses Mut<Account> to test SingleAccountSet trait methods
#[derive(Accounts)]
pub struct TestSingleAccountSetTrait<'info> {
    /// Mut<T> implements SingleAccountSet
    pub data: Mut<Account<'info, TestData>>,
    pub authority: Signer<'info>,
}

// ============================================================================
// Accounts for NEW WRAPPER TESTS - now using wrapper types directly!
// Parser and codegen now support: Owned, Executable, HasOne
// ============================================================================

/// Marker type for this program's ID (for Owned<T, P> wrapper)
pub struct ThisProgram;

impl anchor_lang::Id for ThisProgram {
    fn id() -> Pubkey {
        crate::ID
    }
}

/// Accounts for testing Owned<T, P> wrapper - validates account owner
#[derive(Accounts)]
pub struct TestOwnedWrapper<'info> {
    /// Owned<T, P> validates the account is owned by program P
    pub data: Owned<Account<'info, TestData>, ThisProgram>,
    pub authority: Signer<'info>,
}

/// Accounts for testing Executable<T> wrapper - validates program is executable
#[derive(Accounts)]
pub struct TestExecutableWrapper<'info> {
    /// Executable<T> validates the account is a program
    pub program_account: Executable<UncheckedAccount<'info>>,
    pub authority: Signer<'info>,
}

/// Accounts for testing HasOne<T, Target> wrapper - validates relationships
#[derive(Accounts)]
pub struct TestHasOneWrapper<'info> {
    /// HasOne<T, Target> validates that data.authority == authority.key()
    pub data: HasOne<Account<'info, TestData>, AuthorityTarget>,
    pub authority: Signer<'info>,
}

/// HasOneTarget implementation for authority field
pub struct AuthorityTarget;

impl HasOneTarget<TestData> for AuthorityTarget {
    const FIELD: &'static str = "authority";

    fn target(account: &TestData) -> Pubkey {
        account.authority
    }
}
