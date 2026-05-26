//! Token-2022 CPI helpers and interface re-exports.

extern crate alloc;

use {
    alloc::{string::String, vec, vec::Vec},
    anchor_lang_v2::{CpiContext, CpiHandle, CpiHandleMut, ToCpiAccounts},
    pinocchio::{address::Address, instruction::InstructionAccount},
    solana_program_error::ProgramError,
};

pub use anchor_lang_v2::programs::Token2022;
pub use spl_token_2022_interface::{self as spl_token_2022, extension::ExtensionType, ID};

pub use crate::token_shared::{
    approve, approve_checked, burn, burn_checked, close_account, freeze_account,
    initialize_account, initialize_account3, initialize_mint, initialize_mint2, mint_to,
    mint_to_checked, revoke, set_authority, sync_native, thaw_account, transfer, transfer_checked,
    Approve, ApproveChecked, Burn, BurnChecked, CloseAccount, FreezeAccount, InitializeAccount,
    InitializeAccount3, InitializeMint, InitializeMint2, MintTo, MintToChecked, Revoke,
    SetAuthority, SyncNative, ThawAccount, Transfer, TransferChecked,
};

pub struct GetAccountDataSize<'a> {
    pub mint: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for GetAccountDataSize<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::new(self.mint.address(), false, false)]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint]
    }
}

pub struct InitializeMintCloseAuthority<'a> {
    pub mint: CpiHandleMut<'a>,
}

impl<'a> ToCpiAccounts<'a> for InitializeMintCloseAuthority<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.mint.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint.into()]
    }
}

pub struct InitializeImmutableOwner<'a> {
    pub account: CpiHandleMut<'a>,
}

impl<'a> ToCpiAccounts<'a> for InitializeImmutableOwner<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.account.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.account.into()]
    }
}

pub struct AmountToUiAmount<'a> {
    pub account: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for AmountToUiAmount<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::new(
            self.account.address(),
            false,
            false,
        )]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.account]
    }
}

pub struct UiAmountToAmount<'a> {
    pub account: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for UiAmountToAmount<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::new(
            self.account.address(),
            false,
            false,
        )]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.account]
    }
}

pub struct Reallocate<'a> {
    pub account: CpiHandleMut<'a>,
    pub payer: CpiHandleMut<'a>,
    pub system_program: CpiHandle<'a>,
    pub authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for Reallocate<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.account.address()),
            InstructionAccount::writable_signer(self.payer.address()),
            InstructionAccount::new(self.system_program.address(), false, false),
            InstructionAccount::readonly_signer(self.authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![
            self.account.into(),
            self.payer.into(),
            self.system_program,
            self.authority,
        ]
    }
}

pub struct WithdrawExcessLamports<'a> {
    pub source: CpiHandleMut<'a>,
    pub destination: CpiHandleMut<'a>,
    pub authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for WithdrawExcessLamports<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.source.address()),
            InstructionAccount::writable(self.destination.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.source.into(), self.destination.into(), self.authority]
    }
}

pub struct CreateNativeMint<'a> {
    pub payer: CpiHandleMut<'a>,
    pub native_mint: CpiHandleMut<'a>,
    pub system_program: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for CreateNativeMint<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable_signer(self.payer.address()),
            InstructionAccount::writable(self.native_mint.address()),
            InstructionAccount::new(self.system_program.address(), false, false),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![
            self.payer.into(),
            self.native_mint.into(),
            self.system_program,
        ]
    }
}

pub struct InitializeNonTransferableMint<'a> {
    pub mint: CpiHandleMut<'a>,
}

impl<'a> ToCpiAccounts<'a> for InitializeNonTransferableMint<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.mint.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint.into()]
    }
}

pub struct PermanentDelegateInitialize<'a> {
    pub mint: CpiHandleMut<'a>,
}

impl<'a> ToCpiAccounts<'a> for PermanentDelegateInitialize<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.mint.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint.into()]
    }
}

fn return_data_from(program: &Address) -> Result<Vec<u8>, ProgramError> {
    let (return_program, data) = anchor_lang_v2::solana_program::program::get_return_data()
        .ok_or(ProgramError::InvalidInstructionData)?;
    if return_program.to_bytes().as_slice() != program.as_ref() {
        return Err(ProgramError::IncorrectProgramId);
    }
    Ok(data)
}

pub fn get_account_data_size<'a>(
    ctx: CpiContext<'a, GetAccountDataSize<'a>>,
    extension_types: &[ExtensionType],
) -> Result<u64, ProgramError> {
    let ix = spl_token_2022::instruction::get_account_data_size(
        ctx.program,
        ctx.accounts.mint.address(),
        extension_types,
    )?;
    ctx.invoke_ix(ix)?;
    let data = return_data_from(ctx.program)?;
    let bytes: [u8; 8] = data
        .as_slice()
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    Ok(u64::from_le_bytes(bytes))
}

pub fn initialize_mint_close_authority<'a>(
    ctx: CpiContext<'a, InitializeMintCloseAuthority<'a>>,
    close_authority: Option<&Address>,
) -> Result<(), ProgramError> {
    let ix = spl_token_2022::instruction::initialize_mint_close_authority(
        ctx.program,
        ctx.accounts.mint.address(),
        close_authority,
    )?;
    ctx.invoke_ix(ix)
}

pub fn initialize_immutable_owner<'a>(
    ctx: CpiContext<'a, InitializeImmutableOwner<'a>>,
) -> Result<(), ProgramError> {
    let ix = spl_token_2022::instruction::initialize_immutable_owner(
        ctx.program,
        ctx.accounts.account.address(),
    )?;
    ctx.invoke_ix(ix)
}

pub fn amount_to_ui_amount<'a>(
    ctx: CpiContext<'a, AmountToUiAmount<'a>>,
    amount: u64,
) -> Result<String, ProgramError> {
    let ix = spl_token_2022::instruction::amount_to_ui_amount(
        ctx.program,
        ctx.accounts.account.address(),
        amount,
    )?;
    ctx.invoke_ix(ix)?;
    String::from_utf8(return_data_from(ctx.program)?)
        .map_err(|_| ProgramError::InvalidInstructionData)
}

pub fn ui_amount_to_amount<'a>(
    ctx: CpiContext<'a, UiAmountToAmount<'a>>,
    ui_amount: &str,
) -> Result<u64, ProgramError> {
    let ix = spl_token_2022::instruction::ui_amount_to_amount(
        ctx.program,
        ctx.accounts.account.address(),
        ui_amount,
    )?;
    ctx.invoke_ix(ix)?;
    let data = return_data_from(ctx.program)?;
    let bytes: [u8; 8] = data
        .as_slice()
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    Ok(u64::from_le_bytes(bytes))
}

pub fn reallocate<'a>(
    ctx: CpiContext<'a, Reallocate<'a>>,
    extension_types: &[ExtensionType],
) -> Result<(), ProgramError> {
    let ix = spl_token_2022::instruction::reallocate(
        ctx.program,
        ctx.accounts.account.address(),
        ctx.accounts.payer.address(),
        ctx.accounts.authority.address(),
        &[],
        extension_types,
    )?;
    ctx.invoke_ix(ix)
}

pub fn withdraw_excess_lamports<'a>(
    ctx: CpiContext<'a, WithdrawExcessLamports<'a>>,
) -> Result<(), ProgramError> {
    let ix = spl_token_2022::instruction::withdraw_excess_lamports(
        ctx.program,
        ctx.accounts.source.address(),
        ctx.accounts.destination.address(),
        ctx.accounts.authority.address(),
        &[],
    )?;
    ctx.invoke_ix(ix)
}

pub fn create_native_mint<'a>(
    ctx: CpiContext<'a, CreateNativeMint<'a>>,
) -> Result<(), ProgramError> {
    let ix =
        spl_token_2022::instruction::create_native_mint(ctx.program, ctx.accounts.payer.address())?;
    ctx.invoke_ix(ix)
}

pub fn initialize_non_transferable_mint<'a>(
    ctx: CpiContext<'a, InitializeNonTransferableMint<'a>>,
) -> Result<(), ProgramError> {
    let ix = spl_token_2022::instruction::initialize_non_transferable_mint(
        ctx.program,
        ctx.accounts.mint.address(),
    )?;
    ctx.invoke_ix(ix)
}

pub fn initialize_permanent_delegate<'a>(
    ctx: CpiContext<'a, PermanentDelegateInitialize<'a>>,
    permanent_delegate: &Address,
) -> Result<(), ProgramError> {
    let ix = spl_token_2022::instruction::initialize_permanent_delegate(
        ctx.program,
        ctx.accounts.mint.address(),
        permanent_delegate,
    )?;
    ctx.invoke_ix(ix)
}
