//! HasOne wrapper - enforces a relationship between accounts.
//!
//! This wrapper validates that a field in the wrapped account matches
//! another account's key.

use crate::account_set::SingleAccountSet;
use crate::error::ErrorCode;
use crate::solana_program::account_info::AccountInfo;
use crate::solana_program::instruction::AccountMeta;
use crate::solana_program::pubkey::Pubkey;
use crate::{Accounts, AccountsExit, Key, Result, ToAccountInfos, ToAccountMetas};
use std::collections::BTreeSet;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// Trait for extracting a target pubkey from an account for relationship validation.
///
/// Implement this trait on your account type to define which field should be
/// validated against another account.
///
/// # Example
///
/// ```ignore
/// use anchor_lang::prelude::*;
/// use anchor_lang::account_set::HasOneTarget;
///
/// #[account]
/// pub struct MyData {
///     pub authority: Pubkey,
///     pub value: u64,
/// }
///
/// // Define that we want to validate the 'authority' field
/// pub struct AuthorityTarget;
///
/// impl HasOneTarget<MyData> for AuthorityTarget {
///     const FIELD: &'static str = "authority";
///     fn target(account: &MyData) -> Pubkey {
///         account.authority
///     }
/// }
/// ```
pub trait HasOneTarget<T> {
    /// The field name in the accounts struct to compare against.
    const FIELD: &'static str;
    /// Returns the pubkey from the account that should match the target account.
    fn target(account: &T) -> Pubkey;
}

/// Wrapper that enforces a relationship between accounts.
///
/// # Type Parameters
///
/// - `T`: The inner account type being wrapped
/// - `Target`: The target definition implementing `HasOneTarget`
///
/// # Validation
///
/// At validation time, this wrapper checks that the pubkey returned by
/// `Target::target(account)` matches the expected target key.
///
/// Note: Automatic validation in `#[derive(Accounts)]` requires implementing
/// `HasOneTarget::FIELD` for the target type so the generated Constraints impl can
/// locate the target account. Otherwise, call `validate_has_one()` manually.
///
/// # Example
///
/// ```ignore
/// use anchor_lang::prelude::*;
/// use anchor_lang::account_set::{HasOne, HasOneTarget};
///
/// #[account]
/// pub struct MyData {
///     pub authority: Pubkey,
/// }
///
/// pub struct AuthorityTarget;
/// impl HasOneTarget<MyData> for AuthorityTarget {
///     fn target(account: &MyData) -> Pubkey {
///         account.authority
///     }
/// }
///
/// // In an accounts struct, you would use this with manual validation
/// // or combine with a custom Accounts implementation
/// ```
#[derive(Clone)]
pub struct HasOne<T, Target> {
    inner: T,
    _target: PhantomData<Target>,
}

impl<T, Target> HasOne<T, Target> {
    /// Creates a new `HasOne` wrapper around the given account.
    ///
    /// Note: This does not perform validation. Use `try_from_validated_with_target`
    /// for validated construction.
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            _target: PhantomData,
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

impl<T, Target, D> HasOne<T, Target>
where
    T: std::ops::Deref<Target = D>,
    D: Sized,
    Target: HasOneTarget<D>,
{
    /// Creates a new `HasOne` wrapper, validating that the target field matches.
    ///
    /// # Arguments
    ///
    /// * `inner` - The account to wrap (e.g., Account<'info, MyData>)
    /// * `expected_target` - The pubkey that the target field should match
    ///
    /// # Errors
    ///
    /// Returns `ErrorCode::ConstraintHasOne` if the target field doesn't match.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The wrapper type (e.g., Account<'info, MyData>) that derefs to the data type
    /// - `Target`: The target extractor that works on T::Target (e.g., MyData)
    pub fn try_from_validated_with_target(inner: T, expected_target: &Pubkey) -> Result<Self> {
        let actual_target = Target::target(&inner);
        if actual_target != *expected_target {
            return Err(ErrorCode::ConstraintHasOne.into());
        }
        Ok(Self {
            inner,
            _target: PhantomData,
        })
    }

    /// Returns the target pubkey from the wrapped account.
    pub fn target(&self) -> Pubkey {
        Target::target(&self.inner)
    }
}

// Implement SingleAccountSet by delegation
impl<'info, T: SingleAccountSet<'info>, Target> SingleAccountSet<'info> for HasOne<T, Target> {
    fn account_info(&self) -> &AccountInfo<'info> {
        self.inner.account_info()
    }
}

// Note: HasOne validation cannot happen automatically in try_accounts because:
// 1. Accounts are deserialized one-by-one
// 2. The target account (to compare against) may not be available yet
//
// Validation happens in one of these ways:
// 1. Use `HasOne<T, Target>` and implement `HasOneTarget::FIELD` on `Target` so the generated
//    Constraints impl performs the check.
// 2. Use #[account(has_one = <target>)] so the generated Constraints impl performs the check.
// 3. Call `validate_has_one()` manually in your handler.
//
// Note: Using `HasOne<T, Target>` as a field type generates a has_one check when
// `HasOneTarget::FIELD` is implemented. Otherwise, add the attribute or call manually.

impl<T, Target, D> HasOne<T, Target>
where
    T: std::ops::Deref<Target = D>,
    D: Sized,
    Target: HasOneTarget<D>,
{
    /// Validates the has_one constraint against a target pubkey.
    ///
    /// Call this method after deserialization to validate the relationship.
    ///
    /// # Errors
    ///
    /// Returns `ErrorCode::ConstraintHasOne` if the target field doesn't match.
    pub fn validate_has_one(&self, expected_target: &Pubkey) -> Result<()> {
        let actual_target = Target::target(&self.inner);
        if actual_target != *expected_target {
            return Err(ErrorCode::ConstraintHasOne.into());
        }
        Ok(())
    }
}

// Implement Accounts - deserialization only, validation is deferred
impl<'info, B, T, Target> Accounts<'info, B> for HasOne<T, Target>
where
    T: Accounts<'info, B>,
{
    fn try_accounts(
        program_id: &Pubkey,
        accounts: &mut &'info [AccountInfo<'info>],
        ix_data: &[u8],
        bumps: &mut B,
        reallocs: &mut BTreeSet<Pubkey>,
    ) -> Result<Self> {
        // Deserialize the inner type
        // HasOne validation is deferred - it happens in the Constraints trait
        // or via manual call to validate_has_one()
        let inner = T::try_accounts(program_id, accounts, ix_data, bumps, reallocs)?;

        Ok(Self {
            inner,
            _target: PhantomData,
        })
    }
}

// Implement AccountsExit by delegation
impl<'info, T: AccountsExit<'info>, Target> AccountsExit<'info> for HasOne<T, Target> {
    fn exit(&self, program_id: &Pubkey) -> Result<()> {
        self.inner.exit(program_id)
    }
}

// Implement ToAccountMetas by delegation
impl<T: ToAccountMetas, Target> ToAccountMetas for HasOne<T, Target> {
    fn to_account_metas(&self, is_signer: Option<bool>) -> Vec<AccountMeta> {
        self.inner.to_account_metas(is_signer)
    }
}

// Implement ToAccountInfos by delegation
impl<'info, T: ToAccountInfos<'info>, Target> ToAccountInfos<'info> for HasOne<T, Target> {
    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        self.inner.to_account_infos()
    }
}

// Implement Key by delegation
impl<T: Key, Target> Key for HasOne<T, Target> {
    fn key(&self) -> Pubkey {
        self.inner.key()
    }
}

// Implement AsRef for AccountInfo
impl<'info, T: AsRef<AccountInfo<'info>>, Target> AsRef<AccountInfo<'info>> for HasOne<T, Target> {
    fn as_ref(&self) -> &AccountInfo<'info> {
        self.inner.as_ref()
    }
}

// Implement Deref to inner type for ergonomic access
impl<T, Target> Deref for HasOne<T, Target> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// Implement DerefMut to inner type
impl<T, Target> DerefMut for HasOne<T, Target> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: std::fmt::Debug, Target> std::fmt::Debug for HasOne<T, Target> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HasOne")
            .field("inner", &self.inner)
            .finish()
    }
}
