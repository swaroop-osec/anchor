//! Type validating that the account is the given Program

use crate::error::{Error, ErrorCode};
use crate::solana_program::account_info::AccountInfo;
use crate::solana_program::bpf_loader_upgradeable::{self, UpgradeableLoaderState};
use crate::solana_program::instruction::AccountMeta;
use crate::solana_program::pubkey::Pubkey;
use crate::{
    AccountDeserialize, Accounts, AccountsExit, Id, Key, Result, ToAccountInfos, ToAccountMetas,
};
use std::collections::BTreeSet;
use std::fmt;
use std::marker::PhantomData;
use std::ops::Deref;

/// Type validating that the account is the given Program
///
/// The type has a `programdata_address` function that will return `Option::Some`
/// if the program is owned by the [`BPFUpgradeableLoader`](https://docs.rs/solana-program/latest/solana_program/bpf_loader_upgradeable/index.html)
/// which will contain the `programdata_address` property of the `Program` variant of the [`UpgradeableLoaderState`](https://docs.rs/solana-loader-v3-interface/latest/solana_loader_v3_interface/state/enum.UpgradeableLoaderState.html) enum.
///
/// # Table of Contents
/// - [Basic Functionality](#basic-functionality)
/// - [Generic Program Validation](#generic-program-validation)
/// - [Out of the Box Types](#out-of-the-box-types)
///
/// # Basic Functionality
///
/// For `Program<'info, T>` where T implements Id:
/// Checks:
///
/// - `account_info.key == expected_program`
/// - `account_info.executable == true`
///
/// # Generic Program Validation
///
/// For `Program<'info>` (without type parameter):
/// - Only checks: `account_info.executable == true`
/// - Use this when you only need to verify that an address is executable,
///   without validating against a specific program ID.
///
/// # Example
/// ```ignore
/// #[program]
/// mod my_program {
///     fn set_admin_settings(...){...}
/// }
///
/// #[account]
/// #[derive(Default)]
/// pub struct AdminSettings {
///     ...
/// }
///
/// #[derive(Accounts)]
/// pub struct SetAdminSettings<'info> {
///     #[account(mut, seeds = [b"admin"], bump)]
///     pub admin_settings: Account<'info, AdminSettings>,
///     #[account(constraint = program.programdata_address()? == Some(program_data.key()))]
///     pub program: Program<'info, MyProgram>,
///     #[account(constraint = program_data.upgrade_authority_address == Some(authority.key()))]
///     pub program_data: Account<'info, ProgramData>,
///     pub authority: Signer<'info>,
/// }
/// ```
/// The given program has a function with which the upgrade authority can set admin settings.
///
/// The required constraints are as follows:
///
/// - `program` is the account of the program itself.
///   Its constraint checks that `program_data` is the account that contains the program's upgrade authority.
///   Implicitly, this checks that `program` is a BPFUpgradeable program (`program.programdata_address()?`
///   will be `None` if it's not).
/// - `program_data`'s constraint checks that its upgrade authority is the `authority` account.
/// - Finally, `authority` needs to sign the transaction.
///
/// ## Generic Program Example
/// ```ignore
/// #[derive(Accounts)]
/// pub struct ValidateExecutableProgram<'info> {
///     // Only validates that the provided account is executable
///     pub any_program: Program<'info>,
///     pub authority: Signer<'info>,
/// }
/// ```
///
/// # Out of the Box Types
///
/// Between the [`anchor_lang`](https://docs.rs/anchor-lang/latest/anchor_lang) and [`anchor_spl`](https://docs.rs/anchor_spl/latest/anchor_spl) crates,
/// the following `Program` types are provided out of the box:
///
/// - [`System`](https://docs.rs/anchor-lang/latest/anchor_lang/struct.System.html)
/// - [`AssociatedToken`](https://docs.rs/anchor-spl/latest/anchor_spl/associated_token/struct.AssociatedToken.html)
/// - [`Token`](https://docs.rs/anchor-spl/latest/anchor_spl/token/struct.Token.html)
///
#[derive(Clone)]
pub struct Program<'info, T = ()> {
    info: &'info AccountInfo<'info>,
    _phantom: PhantomData<T>,
}

impl<T: fmt::Debug> fmt::Debug for Program<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Program").field("info", &self.info).finish()
    }
}

impl<'a, T> Program<'a, T> {
    pub(crate) fn new(info: &'a AccountInfo<'a>) -> Program<'a, T> {
        Self {
            info,
            _phantom: PhantomData,
        }
    }

    pub fn programdata_address(&self) -> Result<Option<Pubkey>> {
        if *self.info.owner == bpf_loader_upgradeable::ID {
            let mut data: &[u8] = &self.info.try_borrow_data()?;
            let upgradable_loader_state =
                UpgradeableLoaderState::try_deserialize_unchecked(&mut data)?;

            match upgradable_loader_state {
                UpgradeableLoaderState::Uninitialized
                | UpgradeableLoaderState::Buffer {
                    authority_address: _,
                }
                | UpgradeableLoaderState::ProgramData {
                    slot: _,
                    upgrade_authority_address: _,
                } => {
                    // Unreachable because check in try_from
                    // ensures that program is executable
                    // and therefore a program account.
                    unreachable!()
                }
                UpgradeableLoaderState::Program {
                    programdata_address,
                } => Ok(Some(programdata_address)),
            }
        } else {
            Ok(None)
        }
    }
}

impl<'a, T: Id> TryFrom<&'a AccountInfo<'a>> for Program<'a, T> {
    type Error = Error;
    /// Deserializes the given `info` into a `Program`.
    fn try_from(info: &'a AccountInfo<'a>) -> Result<Self> {
        // Special handling for unit type () - only check executable, not program ID
        let is_unit_type = T::id() == Pubkey::default();

        if !is_unit_type && info.key != &T::id() {
            return Err(Error::from(ErrorCode::InvalidProgramId).with_pubkeys((*info.key, T::id())));
        }
        if !info.executable {
            return Err(ErrorCode::InvalidProgramExecutable.into());
        }
        Ok(Program::new(info))
    }
}

impl<'info, B, T: Id> Accounts<'info, B> for Program<'info, T> {
    #[inline(never)]
    fn try_accounts(
        _program_id: &Pubkey,
        accounts: &mut &'info [AccountInfo<'info>],
        _ix_data: &[u8],
        _bumps: &mut B,
        _reallocs: &mut BTreeSet<Pubkey>,
    ) -> Result<Self> {
        if accounts.is_empty() {
            return Err(ErrorCode::AccountNotEnoughKeys.into());
        }
        let account = &accounts[0];
        *accounts = &accounts[1..];
        Program::try_from(account)
    }
}

impl<T> ToAccountMetas for Program<'_, T> {
    fn to_account_metas(&self, is_signer: Option<bool>) -> Vec<AccountMeta> {
        let is_signer = is_signer.unwrap_or(self.info.is_signer);
        let meta = match self.info.is_writable {
            false => AccountMeta::new_readonly(*self.info.key, is_signer),
            true => AccountMeta::new(*self.info.key, is_signer),
        };
        vec![meta]
    }
}

impl<'info, T> ToAccountInfos<'info> for Program<'info, T> {
    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        vec![self.info.clone()]
    }
}

impl<'info, T> AsRef<AccountInfo<'info>> for Program<'info, T> {
    fn as_ref(&self) -> &AccountInfo<'info> {
        self.info
    }
}

impl<'info, T> Deref for Program<'info, T> {
    type Target = AccountInfo<'info>;

    fn deref(&self) -> &Self::Target {
        self.info
    }
}

impl<'info, T: AccountDeserialize> AccountsExit<'info> for Program<'info, T> {}

impl<T: AccountDeserialize> Key for Program<'_, T> {
    fn key(&self) -> Pubkey {
        *self.info.key
    }
}

// Implement Id trait for unit type to support Program<'info> without type parameter
impl crate::Id for () {
    fn id() -> Pubkey {
        // For generic programs, this should never be called since they don't validate specific program IDs.
        // However, we need to implement it to satisfy the trait bounds.
        // Using a special marker value that indicates "any program"
        Pubkey::default()
    }
}
