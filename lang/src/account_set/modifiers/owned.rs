//! Owned wrapper - enforces that an account is owned by a specific program.

use crate::account_set::SingleAccountSet;
use crate::error::ErrorCode;
use crate::solana_program::account_info::AccountInfo;
use crate::solana_program::instruction::AccountMeta;
use crate::solana_program::pubkey::Pubkey;
use crate::{Accounts, AccountsExit, Id, Key, Result, ToAccountInfos, ToAccountMetas};
use std::collections::BTreeSet;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// Wrapper that enforces an account is owned by a specific program.
///
/// # Type Parameters
///
/// - `T`: The inner account type being wrapped
/// - `P`: The program type that should own this account (must implement `Id`)
///
/// # Validation
///
/// - Checks that the account's owner matches `P::id()`
///
/// # Example
///
/// ```ignore
/// use anchor_lang::prelude::*;
/// use anchor_lang::account_set::Owned;
///
/// #[derive(Accounts)]
/// pub struct VerifyOwnership<'info> {
///     /// CHECK: Owned validates the owner
///     pub token_account: Owned<UncheckedAccount<'info>, TokenProgram>,
/// }
/// ```
#[derive(Clone)]
pub struct Owned<T, P> {
    inner: T,
    _program: PhantomData<P>,
}

impl<T, P> Owned<T, P> {
    /// Creates a new `Owned` wrapper around the given account.
    ///
    /// Note: This does not perform validation. Use `try_from_validated` for validated construction.
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            _program: PhantomData,
        }
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

impl<'info, T: SingleAccountSet<'info>, P: Id> Owned<T, P> {
    /// Creates a new `Owned` wrapper, validating that the account is owned by `P`.
    ///
    /// # Errors
    ///
    /// Returns `ErrorCode::ConstraintOwner` if the account is not owned by `P`.
    pub fn try_from_validated(inner: T) -> Result<Self> {
        let expected_owner = P::id();
        if inner.owner() != expected_owner {
            return Err(ErrorCode::ConstraintOwner.into());
        }
        Ok(Self {
            inner,
            _program: PhantomData,
        })
    }
}

// Implement SingleAccountSet by delegation
impl<'info, T: SingleAccountSet<'info>, P> SingleAccountSet<'info> for Owned<T, P> {
    fn account_info(&self) -> &AccountInfo<'info> {
        self.inner.account_info()
    }
}

// Implement Accounts trait for deserialization
impl<'info, B, T, P> Accounts<'info, B> for Owned<T, P>
where
    T: Accounts<'info, B> + SingleAccountSet<'info>,
    P: Id,
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

        // Then validate the owner constraint
        let expected_owner = P::id();
        if inner.owner() != expected_owner {
            return Err(ErrorCode::ConstraintOwner.into());
        }

        Ok(Self {
            inner,
            _program: PhantomData,
        })
    }
}

// Implement AccountsExit by delegation
impl<'info, T: AccountsExit<'info>, P> AccountsExit<'info> for Owned<T, P> {
    fn exit(&self, program_id: &Pubkey) -> Result<()> {
        self.inner.exit(program_id)
    }
}

// Implement ToAccountMetas by delegation
impl<T: ToAccountMetas, P> ToAccountMetas for Owned<T, P> {
    fn to_account_metas(&self, is_signer: Option<bool>) -> Vec<AccountMeta> {
        self.inner.to_account_metas(is_signer)
    }
}

// Implement ToAccountInfos by delegation
impl<'info, T: ToAccountInfos<'info>, P> ToAccountInfos<'info> for Owned<T, P> {
    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        self.inner.to_account_infos()
    }
}

// Implement Key by delegation
impl<T: Key, P> Key for Owned<T, P> {
    fn key(&self) -> Pubkey {
        self.inner.key()
    }
}

// Implement AsRef for AccountInfo
impl<'info, T: AsRef<AccountInfo<'info>>, P> AsRef<AccountInfo<'info>> for Owned<T, P> {
    fn as_ref(&self) -> &AccountInfo<'info> {
        self.inner.as_ref()
    }
}

// Implement Deref to inner type for ergonomic access
impl<T, P> Deref for Owned<T, P> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// Implement DerefMut to inner type
impl<T, P> DerefMut for Owned<T, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: std::fmt::Debug, P> std::fmt::Debug for Owned<T, P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Owned").field("inner", &self.inner).finish()
    }
}
