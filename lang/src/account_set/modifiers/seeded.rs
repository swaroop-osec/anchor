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

/// Signer seeds with bump for CPI.
///
/// This struct owns the seed data and provides convenient methods
/// for accessing seeds and bump separately or together with `invoke_signed` method.
#[derive(Debug, Clone)]
pub struct SeedsWithBump {
    seeds: Vec<Vec<u8>>,
    bump: u8,
}

impl SeedsWithBump {
    /// Returns the bump seed.
    pub fn bump(&self) -> u8 {
        self.bump
    }

    /// Returns the seeds without the bump.
    pub fn seeds(&self) -> &[Vec<u8>] {
        &self.seeds
    }

    /// Converts to signer seeds format for `invoke_signed`.
    ///
    /// Returns a vector of slices including the bump as the final element.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let swb = pda_account.seeds_with_bump();
    /// invoke_signed(&ix, &accounts, &[&swb.to_signer_seeds()])?;
    /// ```
    pub fn to_signer_seeds(&self) -> Vec<&[u8]> {
        let mut result: Vec<&[u8]> = self.seeds.iter().map(|s| s.as_slice()).collect();
        result.push(std::slice::from_ref(&self.bump));
        result
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
/// - Stores the bump and seeds for later use
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
/// # Bump and Seeds Access
///
/// The bump and seeds are stored and can be accessed for CPI:
///
/// ```ignore
/// let swb = ctx.accounts.config.seeds_with_bump();
/// invoke_signed(&ix, &accounts, &[&swb.to_signer_seeds()])?;
/// ```
#[derive(Clone)]
pub struct Seeded<T, S> {
    inner: T,
    bump: u8,
    seeds: Vec<Vec<u8>>,
    _marker: std::marker::PhantomData<S>,
}

impl<T, S> Seeded<T, S> {
    /// Creates a new `Seeded` wrapper with the given account, bump, and seeds.
    ///
    /// Note: This does not perform validation. Use the `Accounts` implementation
    /// for validated construction.
    pub fn new(inner: T, bump: u8, seeds: Vec<Vec<u8>>) -> Self {
        Self {
            inner,
            bump,
            seeds,
            _marker: std::marker::PhantomData,
        }
    }

    /// Returns the PDA bump seed.
    pub fn bump(&self) -> u8 {
        self.bump
    }

    /// Returns the stored seeds (without bump).
    pub fn seeds(&self) -> &[Vec<u8>] {
        &self.seeds
    }

    /// Returns a `SeedsWithBump` for constructing signer seeds for CPI.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // In a CPI call where the PDA needs to sign:
    /// let swb = ctx.accounts.pda_account.signer_seeds();
    /// invoke_signed(&ix, &accounts, &[&swb.to_signer_seeds()])?;
    /// ```
    pub fn signer_seeds(&self) -> SeedsWithBump {
        SeedsWithBump {
            seeds: self.seeds.clone(),
            bump: self.bump,
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

impl<'info, T: SingleAccountSet<'info>, S: Seeds + Default> Seeded<T, S> {
    /// Validates that the account is a PDA derived from the given seeds.
    ///
    /// This method uses `S::default()` to create the seeds provider,
    /// which works for static seeds. For dynamic seeds, use
    /// `try_from_validated_with_seeds` instead.
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
    /// This is the primary validation method that supports both static and
    /// dynamic seeds. The seeds are stored for later use in CPI signing.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let seeds = MyDynamicSeeds {
    ///     authority: ctx.accounts.authority.key,
    ///     counter: 42,
    /// };
    /// let seeded = Seeded::try_from_validated_with_seeds(account, program_id, &seeds)?;
    /// ```
    pub fn try_from_validated_with_seeds(
        inner: T,
        program_id: &Pubkey,
        seeds_provider: &S,
    ) -> Result<Self> {
        let seed_slices = seeds_provider.seeds();
        let (expected_key, bump) = Pubkey::find_program_address(&seed_slices, program_id);

        if inner.account_info().key != &expected_key {
            return Err(crate::error::Error::from(ErrorCode::ConstraintSeeds)
                .with_pubkeys((*inner.account_info().key, expected_key)));
        }

        // Store seeds as owned data for later use in CPI
        let seeds: Vec<Vec<u8>> = seed_slices.into_iter().map(|s| s.to_vec()).collect();

        Ok(Self {
            inner,
            bump,
            seeds,
            _marker: std::marker::PhantomData,
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
            .field("seeds", &self.seeds)
            .finish()
    }
}
