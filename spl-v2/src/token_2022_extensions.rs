//! Token-2022 extension CPI helpers.

extern crate alloc;

use {
    alloc::{vec, vec::Vec},
    anchor_lang_v2::{CpiContext, CpiHandle, ToCpiAccounts},
    pinocchio::instruction::InstructionAccount,
    solana_address::Address,
};

const EXT_CPI_GUARD: u8 = 34;
const EXT_GROUP_POINTER: u8 = 40;
const EXT_GROUP_MEMBER_POINTER: u8 = 41;
const EXT_PAUSABLE: u8 = 44;

const DISC_INITIALIZE: u8 = 0;
const DISC_UPDATE: u8 = 1;
const DISC_RESUME: u8 = 2;

const TOKEN_METADATA_REMOVE_KEY_DISCRIMINATOR: [u8; 8] =
    [0xea, 0x12, 0x20, 0x38, 0x59, 0x8d, 0x25, 0xb5];

pub struct CpiGuard<'a> {
    pub account: CpiHandle<'a>,
    pub owner: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for CpiGuard<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.account.address()),
            InstructionAccount::readonly_signer(self.owner.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.account, self.owner]
    }
}

pub struct GroupPointerInitialize<'a> {
    pub mint: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for GroupPointerInitialize<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.mint.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint]
    }
}

pub struct GroupPointerUpdate<'a> {
    pub mint: CpiHandle<'a>,
    pub authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for GroupPointerUpdate<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.mint.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint, self.authority]
    }
}

pub struct GroupMemberPointerInitialize<'a> {
    pub mint: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for GroupMemberPointerInitialize<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.mint.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint]
    }
}

pub struct GroupMemberPointerUpdate<'a> {
    pub mint: CpiHandle<'a>,
    pub authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for GroupMemberPointerUpdate<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.mint.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint, self.authority]
    }
}

pub struct PausableInitialize<'a> {
    pub mint: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for PausableInitialize<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.mint.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint]
    }
}

pub struct PausableToggle<'a> {
    pub mint: CpiHandle<'a>,
    pub authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for PausableToggle<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.mint.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint, self.authority]
    }
}

pub struct TokenMetadataRemoveKey<'a> {
    pub metadata: CpiHandle<'a>,
    pub update_authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for TokenMetadataRemoveKey<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.metadata.address()),
            InstructionAccount::readonly_signer(self.update_authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.metadata, self.update_authority]
    }
}

fn encode_optional_address(address: Option<&Address>) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    if let Some(address) = address {
        bytes.copy_from_slice(address.as_ref());
    }
    bytes
}

fn encode_group_pointer_initialize(
    authority: Option<&Address>,
    group_address: Option<&Address>,
) -> [u8; 66] {
    let mut data = [0u8; 66];
    data[0] = EXT_GROUP_POINTER;
    data[1] = DISC_INITIALIZE;
    data[2..34].copy_from_slice(&encode_optional_address(authority));
    data[34..66].copy_from_slice(&encode_optional_address(group_address));
    data
}

fn encode_group_pointer_update(group_address: Option<&Address>) -> [u8; 34] {
    let mut data = [0u8; 34];
    data[0] = EXT_GROUP_POINTER;
    data[1] = DISC_UPDATE;
    data[2..34].copy_from_slice(&encode_optional_address(group_address));
    data
}

fn encode_group_member_pointer_initialize(
    authority: Option<&Address>,
    member_address: Option<&Address>,
) -> [u8; 66] {
    let mut data = [0u8; 66];
    data[0] = EXT_GROUP_MEMBER_POINTER;
    data[1] = DISC_INITIALIZE;
    data[2..34].copy_from_slice(&encode_optional_address(authority));
    data[34..66].copy_from_slice(&encode_optional_address(member_address));
    data
}

fn encode_group_member_pointer_update(member_address: Option<&Address>) -> [u8; 34] {
    let mut data = [0u8; 34];
    data[0] = EXT_GROUP_MEMBER_POINTER;
    data[1] = DISC_UPDATE;
    data[2..34].copy_from_slice(&encode_optional_address(member_address));
    data
}

fn encode_pausable_initialize(authority: &Address) -> [u8; 34] {
    let mut data = [0u8; 34];
    data[0] = EXT_PAUSABLE;
    data[1] = DISC_INITIALIZE;
    data[2..34].copy_from_slice(authority.as_ref());
    data
}

fn encode_token_metadata_remove_key(key: &str, idempotent: bool) -> Vec<u8> {
    let mut data = Vec::with_capacity(8 + 1 + 4 + key.len());
    data.extend_from_slice(&TOKEN_METADATA_REMOVE_KEY_DISCRIMINATOR);
    data.push(u8::from(idempotent));
    data.extend_from_slice(&(key.len() as u32).to_le_bytes());
    data.extend_from_slice(key.as_bytes());
    data
}

#[deprecated(
    note = "Token-2022 rejects CPI-initiated toggling of CPI Guard with CpiGuardSettingsLocked."
)]
pub fn cpi_guard_enable<'a>(ctx: CpiContext<'a, CpiGuard<'a>>) {
    ctx.invoke(&[EXT_CPI_GUARD, DISC_INITIALIZE]);
}

#[deprecated(
    note = "Token-2022 rejects CPI-initiated toggling of CPI Guard with CpiGuardSettingsLocked."
)]
pub fn cpi_guard_disable<'a>(ctx: CpiContext<'a, CpiGuard<'a>>) {
    ctx.invoke(&[EXT_CPI_GUARD, DISC_UPDATE]);
}

pub fn group_pointer_initialize<'a>(
    ctx: CpiContext<'a, GroupPointerInitialize<'a>>,
    authority: Option<&Address>,
    group_address: Option<&Address>,
) {
    ctx.invoke(&encode_group_pointer_initialize(authority, group_address));
}

pub fn group_pointer_update<'a>(
    ctx: CpiContext<'a, GroupPointerUpdate<'a>>,
    group_address: Option<&Address>,
) {
    ctx.invoke(&encode_group_pointer_update(group_address));
}

pub fn group_member_pointer_initialize<'a>(
    ctx: CpiContext<'a, GroupMemberPointerInitialize<'a>>,
    authority: Option<&Address>,
    member_address: Option<&Address>,
) {
    ctx.invoke(&encode_group_member_pointer_initialize(
        authority,
        member_address,
    ));
}

pub fn group_member_pointer_update<'a>(
    ctx: CpiContext<'a, GroupMemberPointerUpdate<'a>>,
    member_address: Option<&Address>,
) {
    ctx.invoke(&encode_group_member_pointer_update(member_address));
}

pub fn pausable_initialize<'a>(ctx: CpiContext<'a, PausableInitialize<'a>>, authority: &Address) {
    ctx.invoke(&encode_pausable_initialize(authority));
}

pub fn pausable_pause<'a>(ctx: CpiContext<'a, PausableToggle<'a>>) {
    ctx.invoke(&[EXT_PAUSABLE, DISC_UPDATE]);
}

pub fn pausable_resume<'a>(ctx: CpiContext<'a, PausableToggle<'a>>) {
    ctx.invoke(&[EXT_PAUSABLE, DISC_RESUME]);
}

pub fn token_metadata_remove_key<'a>(
    ctx: CpiContext<'a, TokenMetadataRemoveKey<'a>>,
    key: &str,
    idempotent: bool,
) {
    ctx.invoke(&encode_token_metadata_remove_key(key, idempotent));
}
