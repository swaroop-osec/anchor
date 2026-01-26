//! SingleAccountSet implementations for existing Anchor account types.

use super::SingleAccountSet;
use crate::accounts::account::Account;
use crate::accounts::program::Program;
use crate::accounts::signer::Signer;
use crate::accounts::system_account::SystemAccount;
use crate::accounts::unchecked_account::UncheckedAccount;
use crate::solana_program::account_info::AccountInfo;
use crate::{AccountDeserialize, AccountSerialize, Id};

// Implement SingleAccountSet for Account<'info, T>
impl<'info, T: AccountSerialize + AccountDeserialize + Clone> SingleAccountSet<'info>
    for Account<'info, T>
{
    fn account_info(&self) -> &AccountInfo<'info> {
        self.as_ref()
    }
}

// Implement SingleAccountSet for Signer<'info>
impl<'info> SingleAccountSet<'info> for Signer<'info> {
    fn account_info(&self) -> &AccountInfo<'info> {
        self.as_ref()
    }
}

// Implement SingleAccountSet for SystemAccount<'info>
impl<'info> SingleAccountSet<'info> for SystemAccount<'info> {
    fn account_info(&self) -> &AccountInfo<'info> {
        self.as_ref()
    }
}

// Implement SingleAccountSet for UncheckedAccount<'info>
impl<'info> SingleAccountSet<'info> for UncheckedAccount<'info> {
    fn account_info(&self) -> &AccountInfo<'info> {
        self.as_ref()
    }
}

// Implement SingleAccountSet for Program<'info, T>
impl<'info, T: Id> SingleAccountSet<'info> for Program<'info, T> {
    fn account_info(&self) -> &AccountInfo<'info> {
        self.as_ref()
    }
}

#[cfg(feature = "lazy-account")]
impl<'info, T: crate::AccountDeserialize + crate::AccountSerialize + crate::Owner + Clone>
    SingleAccountSet<'info> for crate::accounts::lazy_account::LazyAccount<'info, T>
{
    fn account_info(&self) -> &AccountInfo<'info> {
        self.as_ref()
    }
}
