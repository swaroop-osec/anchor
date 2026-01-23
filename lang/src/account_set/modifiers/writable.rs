//! Mut wrapper - enforces that an account is writable.

use crate::account_set::SingleAccountSet;
use crate::error::ErrorCode;
use crate::solana_program::account_info::AccountInfo;
use crate::solana_program::instruction::AccountMeta;
use crate::solana_program::pubkey::Pubkey;
use crate::{Accounts, AccountsExit, Key, Result, ToAccountInfos, ToAccountMetas};
use std::collections::BTreeSet;
use std::ops::{Deref, DerefMut};

/// Wrapper that enforces an account is marked as writable.
///
/// # Validation
///
/// - Checks that the account's `is_writable` flag is `true`
///
/// # Example
///
/// ```ignore
/// use anchor_lang::prelude::*;
/// use anchor_lang::account_set::Mut;
///
/// #[derive(Accounts)]
/// pub struct UpdateData<'info> {
///     // This account must be writable
///     pub data: Mut<Account<'info, MyData>>,
/// }
/// ```
///
/// # Composition
///
/// Can be combined with other wrappers:
///
/// ```ignore
/// // PDA AND writable
/// Mut<Seeded<Account<'info, MyData>, MySeeds>>
/// ```
#[derive(Clone)]
pub struct Mut<T> {
    inner: T,
}

impl<T> Mut<T> {
    /// Creates a new `Mut` wrapper around the given account.
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

impl<'info, T: SingleAccountSet<'info>> Mut<T> {
    /// Creates a new `Mut` wrapper, validating that the account is writable.
    ///
    /// # Errors
    ///
    /// Returns `ErrorCode::ConstraintMut` if the account is not writable.
    pub fn try_from_validated(inner: T) -> Result<Self> {
        if !inner.is_writable() {
            return Err(ErrorCode::ConstraintMut.into());
        }
        Ok(Self { inner })
    }
}

// Implement SingleAccountSet by delegation
impl<'info, T: SingleAccountSet<'info>> SingleAccountSet<'info> for Mut<T> {
    fn account_info(&self) -> &AccountInfo<'info> {
        self.inner.account_info()
    }
}

// Implement Accounts trait for deserialization
impl<'info, B, T> Accounts<'info, B> for Mut<T>
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

        // Then validate the writable constraint
        // Use ConstraintMut to match the error code from #[account(mut)]
        if !inner.is_writable() {
            return Err(ErrorCode::ConstraintMut.into());
        }

        Ok(Self { inner })
    }
}

// Implement AccountsExit by delegation
impl<'info, T: AccountsExit<'info>> AccountsExit<'info> for Mut<T> {
    fn exit(&self, program_id: &Pubkey) -> Result<()> {
        self.inner.exit(program_id)
    }
}

// Implement ToAccountMetas - always marks as writable
impl<T: ToAccountMetas> ToAccountMetas for Mut<T> {
    fn to_account_metas(&self, is_signer: Option<bool>) -> Vec<AccountMeta> {
        // Get inner metas and force writable
        self.inner
            .to_account_metas(is_signer)
            .into_iter()
            .map(|meta| AccountMeta {
                pubkey: meta.pubkey,
                is_signer: meta.is_signer,
                is_writable: true, // Force writable
            })
            .collect()
    }
}

// Implement ToAccountInfos by delegation
impl<'info, T: ToAccountInfos<'info>> ToAccountInfos<'info> for Mut<T> {
    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        self.inner.to_account_infos()
    }
}

// Implement Key by delegation
impl<T: Key> Key for Mut<T> {
    fn key(&self) -> Pubkey {
        self.inner.key()
    }
}

// Implement AsRef for AccountInfo
impl<'info, T: AsRef<AccountInfo<'info>>> AsRef<AccountInfo<'info>> for Mut<T> {
    fn as_ref(&self) -> &AccountInfo<'info> {
        self.inner.as_ref()
    }
}

// Implement Deref to inner type for ergonomic access
impl<T> Deref for Mut<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// Implement DerefMut to inner type
impl<T> DerefMut for Mut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Mut<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mut").field("inner", &self.inner).finish()
    }
}
