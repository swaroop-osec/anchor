//! Single account set trait for individual account operations.

use crate::error::ErrorCode;
use crate::solana_program::account_info::AccountInfo;
use crate::solana_program::pubkey::Pubkey;
use crate::Result;

/// Trait for types representing a single account with basic validation helpers.
///
/// This trait provides a unified interface for accessing account info and
/// performing common validation checks. All account wrapper types should
/// implement this trait.
///
/// # Example
///
/// ```ignore
/// use anchor_lang::prelude::*;
/// use anchor_lang::account_set::SingleAccountSet;
///
/// fn check_account<'info, T: SingleAccountSet<'info>>(account: &T) -> Result<()> {
///     // Access the underlying account info
///     let info = account.account_info();
///
///     // Use helper methods for validation
///     account.check_signer()?;
///     account.check_writable()?;
///
///     Ok(())
/// }
/// ```
pub trait SingleAccountSet<'info> {
    /// Returns the underlying `AccountInfo` for this account.
    fn account_info(&self) -> &AccountInfo<'info>;

    /// Returns the public key of this account.
    fn pubkey(&self) -> Pubkey {
        *self.account_info().key
    }

    /// Returns whether this account signed the transaction.
    fn is_signer(&self) -> bool {
        self.account_info().is_signer
    }

    /// Returns whether this account is writable.
    fn is_writable(&self) -> bool {
        self.account_info().is_writable
    }

    /// Returns the owner program of this account.
    fn owner(&self) -> Pubkey {
        *self.account_info().owner
    }

    /// Returns the lamports balance of this account.
    fn lamports(&self) -> u64 {
        self.account_info().lamports()
    }

    /// Checks that this account signed the transaction.
    ///
    /// # Errors
    ///
    /// Returns `ErrorCode::AccountNotSigner` if the account did not sign.
    fn check_signer(&self) -> Result<()> {
        if !self.is_signer() {
            return Err(ErrorCode::AccountNotSigner.into());
        }
        Ok(())
    }

    /// Checks that this account is marked as writable.
    ///
    /// # Errors
    ///
    /// Returns `ErrorCode::AccountNotMutable` if the account is not writable.
    fn check_writable(&self) -> Result<()> {
        if !self.is_writable() {
            return Err(ErrorCode::AccountNotMutable.into());
        }
        Ok(())
    }

    /// Checks that this account is owned by the expected program.
    ///
    /// # Errors
    ///
    /// Returns `ErrorCode::AccountOwnedByWrongProgram` if the owner doesn't match.
    fn check_owner(&self, expected_owner: &Pubkey) -> Result<()> {
        if self.owner() != *expected_owner {
            return Err(
                crate::error::Error::from(ErrorCode::AccountOwnedByWrongProgram)
                    .with_pubkeys((self.owner(), *expected_owner)),
            );
        }
        Ok(())
    }

    /// Checks that this account's key matches the expected key.
    ///
    /// # Errors
    ///
    /// Returns `ErrorCode::ConstraintAddress` if the keys don't match.
    fn check_key(&self, expected_key: &Pubkey) -> Result<()> {
        if self.pubkey() != *expected_key {
            return Err(crate::error::Error::from(ErrorCode::ConstraintAddress)
                .with_pubkeys((self.pubkey(), *expected_key)));
        }
        Ok(())
    }
}

// Implement SingleAccountSet for AccountInfo directly
impl<'info> SingleAccountSet<'info> for AccountInfo<'info> {
    fn account_info(&self) -> &AccountInfo<'info> {
        self
    }
}

// Implement SingleAccountSet for references to AccountInfo
impl<'info> SingleAccountSet<'info> for &AccountInfo<'info> {
    fn account_info(&self) -> &AccountInfo<'info> {
        self
    }
}
