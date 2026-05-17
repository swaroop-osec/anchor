//! Token-2022 CPI helpers that are not part of the legacy Token program.

extern crate alloc;

use {
    alloc::{vec, vec::Vec},
    anchor_lang_v2::{CpiContext, CpiHandle, ToCpiAccounts},
    pinocchio::instruction::InstructionAccount,
};

/// Token-2022 account extension types used by `reallocate`.
///
/// The discriminants match v1's
/// `spl_token_2022_interface::extension::ExtensionType`, not the Token-2022
/// instruction extension discriminators.
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtensionType {
    Uninitialized = 0,
    TransferFeeConfig = 1,
    TransferFeeAmount = 2,
    MintCloseAuthority = 3,
    ConfidentialTransferMint = 4,
    ConfidentialTransferAccount = 5,
    DefaultAccountState = 6,
    ImmutableOwner = 7,
    MemoTransfer = 8,
    NonTransferable = 9,
    InterestBearingConfig = 10,
    CpiGuard = 11,
    PermanentDelegate = 12,
    NonTransferableAccount = 13,
    TransferHook = 14,
    TransferHookAccount = 15,
    ConfidentialTransferFeeConfig = 16,
    ConfidentialTransferFeeAmount = 17,
    MetadataPointer = 18,
    TokenMetadata = 19,
    GroupPointer = 20,
    TokenGroup = 21,
    GroupMemberPointer = 22,
    TokenGroupMember = 23,
    ConfidentialMintBurn = 24,
    ScaledUiAmount = 25,
    Pausable = 26,
    PausableAccount = 27,
}

/// Minimal v1-compatible path for callers that name
/// `token_2022::spl_token_2022::extension::ExtensionType`.
pub mod spl_token_2022 {
    pub mod extension {
        pub use super::super::ExtensionType;
    }
}

pub struct CreateNativeMint<'a> {
    pub payer: CpiHandle<'a>,
    pub native_mint: CpiHandle<'a>,
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
        vec![self.payer, self.native_mint, self.system_program]
    }
}

pub struct InitializeNonTransferableMint<'a> {
    pub mint: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for InitializeNonTransferableMint<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.mint.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint]
    }
}

pub struct Reallocate<'a> {
    pub account: CpiHandle<'a>,
    pub payer: CpiHandle<'a>,
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
            self.account,
            self.payer,
            self.system_program,
            self.authority,
        ]
    }
}

pub struct WithdrawExcessLamports<'a> {
    pub source: CpiHandle<'a>,
    pub destination: CpiHandle<'a>,
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
        vec![self.source, self.destination, self.authority]
    }
}

const DISC_REALLOCATE: u8 = 29;
const DISC_CREATE_NATIVE_MINT: u8 = 31;
const DISC_INITIALIZE_NON_TRANSFERABLE_MINT: u8 = 32;
const DISC_WITHDRAW_EXCESS_LAMPORTS: u8 = 38;

fn encode_reallocate_ix(extension_types: &[ExtensionType]) -> Vec<u8> {
    let mut data = Vec::with_capacity(1 + extension_types.len() * 2);
    data.push(DISC_REALLOCATE);
    for extension_type in extension_types {
        data.extend_from_slice(&(*extension_type as u16).to_le_bytes());
    }
    data
}

pub fn create_native_mint<'a>(ctx: CpiContext<'a, CreateNativeMint<'a>>) {
    ctx.invoke(&[DISC_CREATE_NATIVE_MINT]);
}

pub fn initialize_non_transferable_mint<'a>(
    ctx: CpiContext<'a, InitializeNonTransferableMint<'a>>,
) {
    ctx.invoke(&[DISC_INITIALIZE_NON_TRANSFERABLE_MINT]);
}

pub fn reallocate<'a>(ctx: CpiContext<'a, Reallocate<'a>>, extension_types: &[ExtensionType]) {
    ctx.invoke(&encode_reallocate_ix(extension_types));
}

pub fn withdraw_excess_lamports<'a>(ctx: CpiContext<'a, WithdrawExcessLamports<'a>>) {
    ctx.invoke(&[DISC_WITHDRAW_EXCESS_LAMPORTS]);
}
