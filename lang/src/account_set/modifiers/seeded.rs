//! Seeded wrapper - enforces PDA validation and captures bump.

use crate::account_set::SingleAccountSet;
use crate::error::ErrorCode;
use crate::solana_program::account_info::AccountInfo;
use crate::solana_program::instruction::AccountMeta;
use crate::solana_program::pubkey::Pubkey;
use crate::{Accounts, AccountsExit, Key, Result, ToAccountInfos, ToAccountMetas};
use std::collections::BTreeSet;
use std::ops::{Deref, DerefMut};

/// Trait for types that provide PDA seeds.
///
/// Implement this trait to define the seeds used for PDA derivation.
///
/// # Example
///
/// ```ignore
/// use anchor_lang::account_set::Seeds;
///
/// pub struct MySeeds<'a> {
///     pub authority: &'a Pubkey,
///     pub counter: u64,
/// }
///
/// impl<'a> Seeds for MySeeds<'a> {
///     fn seeds(&self) -> Vec<&[u8]> {
///         vec![
///             b"my_account",
///             self.authority.as_ref(),
///             &self.counter.to_le_bytes(),
///         ]
///     }
/// }
/// ```
pub trait Seeds {
    /// Returns the seed slices for PDA derivation.
    ///
    /// These seeds are used with `Pubkey::find_program_address` to derive
    /// the PDA and validate the account key.
    fn seeds(&self) -> Vec<&[u8]>;
}

/// Seeds with an associated bump.
///
/// This is useful when you already know the bump and want to construct
/// signer seeds for CPI.
pub struct SeedsWithBump<'a> {
    seeds: Vec<&'a [u8]>,
    bump: u8,
}

impl<'a> SeedsWithBump<'a> {
    /// Creates a new `SeedsWithBump` from seeds and a bump.
    pub fn new(seeds: Vec<&'a [u8]>, bump: u8) -> Self {
        Self { seeds, bump }
    }

    /// Returns the bump seed.
    pub fn bump(&self) -> u8 {
        self.bump
    }

    /// Returns the seeds without the bump.
    pub fn seeds(&self) -> &[&'a [u8]] {
        &self.seeds
    }

    /// Returns the signer seeds without the bump.
    ///
    /// Callers should append the bump (e.g., `&[self.bump()]`) when constructing
    /// the final signer seeds for `invoke_signed`.
    pub fn signer_seeds(&self) -> Vec<&[u8]> {
        self.seeds.clone()
    }
}

impl<'a> Seeds for SeedsWithBump<'a> {
    fn seeds(&self) -> Vec<&[u8]> {
        self.seeds.clone()
    }
}

/// Wrapper that validates an account is a PDA derived from specific seeds.
///
/// # Type Parameters
///
/// - `T`: The inner account type
/// - `S`: The seeds type implementing the `Seeds` trait
///
/// # Validation
///
/// - Derives the PDA using `Pubkey::find_program_address`
/// - Checks that the account key matches the derived PDA
/// - Stores the bump for later use
///
/// # Example
///
/// ```ignore
/// use anchor_lang::prelude::*;
/// use anchor_lang::account_set::{Seeded, Seeds};
///
/// pub struct ConfigSeeds;
///
/// impl Seeds for ConfigSeeds {
///     fn seeds(&self) -> Vec<&[u8]> {
///         vec![b"config"]
///     }
/// }
///
/// #[derive(Accounts)]
/// pub struct ReadConfig<'info> {
///     pub config: Seeded<Account<'info, Config>, ConfigSeeds>,
/// }
/// ```
///
/// # Bump Access
///
/// The bump is stored and can be accessed for CPI:
///
/// ```ignore
/// let bump = ctx.accounts.config.bump();
/// let seeds = &[b"config", &[bump]];
/// ```
#[derive(Clone)]
pub struct Seeded<T, S> {
    inner: T,
    bump: u8,
    _seeds: std::marker::PhantomData<S>,
}

impl<T, S> Seeded<T, S> {
    /// Creates a new `Seeded` wrapper with the given account and bump.
    ///
    /// Note: This does not perform validation. Use the `Accounts` implementation
    /// for validated construction.
    pub fn new(inner: T, bump: u8) -> Self {
        Self {
            inner,
            bump,
            _seeds: std::marker::PhantomData,
        }
    }

    /// Returns the PDA bump seed.
    pub fn bump(&self) -> u8 {
        self.bump
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

impl<'info, T: SingleAccountSet<'info>, S: Seeds + Default> Seeded<T, S> {
    /// Validates that the account is a PDA derived from the given seeds.
    ///
    /// # Errors
    ///
    /// Returns `ErrorCode::ConstraintSeeds` if the account key doesn't match
    /// the derived PDA.
    pub fn try_from_validated(inner: T, program_id: &Pubkey) -> Result<Self> {
        let seeds_provider = S::default();
        Self::try_from_validated_with_seeds(inner, program_id, &seeds_provider)
    }
}

impl<'info, T: SingleAccountSet<'info>, S: Seeds> Seeded<T, S> {
    /// Validates that the account is a PDA derived from the provided seeds.
    ///
    /// This helper allows runtime seed providers and avoids the `Default` bound.
    pub fn try_from_validated_with_seeds(
        inner: T,
        program_id: &Pubkey,
        seeds_provider: &S,
    ) -> Result<Self> {
        let seeds = seeds_provider.seeds();
        let (expected_key, bump) = Pubkey::find_program_address(&seeds, program_id);

        if inner.account_info().key != &expected_key {
            return Err(crate::error::Error::from(ErrorCode::ConstraintSeeds)
                .with_pubkeys((*inner.account_info().key, expected_key)));
        }

        Ok(Self {
            inner,
            bump,
            _seeds: std::marker::PhantomData,
        })
    }
}

// Implement SingleAccountSet by delegation
impl<'info, T: SingleAccountSet<'info>, S> SingleAccountSet<'info> for Seeded<T, S> {
    fn account_info(&self) -> &AccountInfo<'info> {
        self.inner.account_info()
    }
}

// Implement Accounts trait for deserialization
impl<'info, B, T, S> Accounts<'info, B> for Seeded<T, S>
where
    T: Accounts<'info, B> + SingleAccountSet<'info>,
    S: Seeds + Default,
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

        // Get seeds and validate PDA
        let seeds_provider = S::default();
        Seeded::try_from_validated_with_seeds(inner, program_id, &seeds_provider)
    }
}

// Implement AccountsExit by delegation
impl<'info, T: AccountsExit<'info>, S> AccountsExit<'info> for Seeded<T, S> {
    fn exit(&self, program_id: &Pubkey) -> Result<()> {
        self.inner.exit(program_id)
    }
}

// Implement ToAccountMetas by delegation
impl<T: ToAccountMetas, S> ToAccountMetas for Seeded<T, S> {
    fn to_account_metas(&self, is_signer: Option<bool>) -> Vec<AccountMeta> {
        self.inner.to_account_metas(is_signer)
    }
}

// Implement ToAccountInfos by delegation
impl<'info, T: ToAccountInfos<'info>, S> ToAccountInfos<'info> for Seeded<T, S> {
    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        self.inner.to_account_infos()
    }
}

// Implement Key by delegation
impl<T: Key, S> Key for Seeded<T, S> {
    fn key(&self) -> Pubkey {
        self.inner.key()
    }
}

// Implement AsRef for AccountInfo
impl<'info, T: AsRef<AccountInfo<'info>>, S> AsRef<AccountInfo<'info>> for Seeded<T, S> {
    fn as_ref(&self) -> &AccountInfo<'info> {
        self.inner.as_ref()
    }
}

// Implement Deref to inner type for ergonomic access
impl<T, S> Deref for Seeded<T, S> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// Implement DerefMut to inner type
impl<T, S> DerefMut for Seeded<T, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: std::fmt::Debug, S> std::fmt::Debug for Seeded<T, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Seeded")
            .field("inner", &self.inner)
            .field("bump", &self.bump)
            .finish()
    }
}
