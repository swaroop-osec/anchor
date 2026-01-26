//! Executable wrapper - enforces that an account is an executable program.

use crate::account_set::SingleAccountSet;
use crate::error::ErrorCode;
use crate::solana_program::account_info::AccountInfo;
use crate::solana_program::instruction::AccountMeta;
use crate::solana_program::pubkey::Pubkey;
use crate::{Accounts, AccountsExit, Key, Result, ToAccountInfos, ToAccountMetas};
use std::collections::BTreeSet;
use std::ops::{Deref, DerefMut};

/// Wrapper that enforces an account is executable (i.e., is a program).
///
/// # Validation
///
/// - Checks that the account's `executable` flag is `true`
///
/// # Example
///
/// ```ignore
/// use anchor_lang::prelude::*;
/// use anchor_lang::account_set::Executable;
///
/// #[derive(Accounts)]
/// pub struct InvokeProgram<'info> {
///     /// CHECK: Executable validates this is a program
///     pub target_program: Executable<UncheckedAccount<'info>>,
/// }
/// ```
#[derive(Clone)]
pub struct Executable<T> {
    inner: T,
}

impl<T> Executable<T> {
    /// Creates a new `Executable` wrapper around the given account.
    ///
    /// Note: This does not perform validation. Use `try_from_validated` for validated construction.
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    /// Returns a reference to the inner account.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Returns a mutable reference to the inner account.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Consumes the wrapper and returns the inner account.
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<'info, T: SingleAccountSet<'info>> Executable<T> {
    /// Creates a new `Executable` wrapper, validating that the account is executable.
    ///
    /// # Errors
    ///
    /// Returns `ErrorCode::ConstraintExecutable` if the account is not executable.
    pub fn try_from_validated(inner: T) -> Result<Self> {
        if !inner.account_info().executable {
            return Err(ErrorCode::ConstraintExecutable.into());
        }
        Ok(Self { inner })
    }
}

// Implement SingleAccountSet by delegation
impl<'info, T: SingleAccountSet<'info>> SingleAccountSet<'info> for Executable<T> {
    fn account_info(&self) -> &AccountInfo<'info> {
        self.inner.account_info()
    }
}

// Implement Accounts trait for deserialization
impl<'info, B, T> Accounts<'info, B> for Executable<T>
where
    T: Accounts<'info, B> + SingleAccountSet<'info>,
{
    fn try_accounts(
        program_id: &Pubkey,
        accounts: &mut &'info [AccountInfo<'info>],
        ix_data: &[u8],
        bumps: &mut B,
        reallocs: &mut BTreeSet<Pubkey>,
    ) -> Result<Self> {
        // First, deserialize the inner type
        let inner = T::try_accounts(program_id, accounts, ix_data, bumps, reallocs)?;

        // Then validate the executable constraint
        if !inner.account_info().executable {
            return Err(ErrorCode::ConstraintExecutable.into());
        }

        Ok(Self { inner })
    }
}

// Implement AccountsExit by delegation
impl<'info, T: AccountsExit<'info>> AccountsExit<'info> for Executable<T> {
    fn exit(&self, program_id: &Pubkey) -> Result<()> {
        self.inner.exit(program_id)
    }
}

// Implement ToAccountMetas by delegation
impl<T: ToAccountMetas> ToAccountMetas for Executable<T> {
    fn to_account_metas(&self, is_signer: Option<bool>) -> Vec<AccountMeta> {
        self.inner.to_account_metas(is_signer)
    }
}

// Implement ToAccountInfos by delegation
impl<'info, T: ToAccountInfos<'info>> ToAccountInfos<'info> for Executable<T> {
    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        self.inner.to_account_infos()
    }
}

// Implement Key by delegation
impl<T: Key> Key for Executable<T> {
    fn key(&self) -> Pubkey {
        self.inner.key()
    }
}

// Implement AsRef for AccountInfo
impl<'info, T: AsRef<AccountInfo<'info>>> AsRef<AccountInfo<'info>> for Executable<T> {
    fn as_ref(&self) -> &AccountInfo<'info> {
        self.inner.as_ref()
    }
}

// Implement Deref to inner type for ergonomic access
impl<T> Deref for Executable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// Implement DerefMut to inner type
impl<T> DerefMut for Executable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Executable<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Executable")
            .field("inner", &self.inner)
            .finish()
    }
}
