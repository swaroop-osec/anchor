//! Token-2022 extension CPI helpers.

extern crate alloc;

use {
    crate::token_2022::spl_token_2022,
    alloc::{string::String, vec, vec::Vec},
    anchor_lang_v2::{programs::Token2022, CpiContext, CpiHandle, Id, ToCpiAccounts},
    pinocchio::{address::Address, instruction::InstructionAccount},
    solana_instruction::Instruction,
    solana_program_error::ProgramError,
    solana_pubkey::Pubkey,
    spl_pod::optional_keys::OptionalNonZeroPubkey,
    spl_token_metadata_interface::state::Field,
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

fn assert_token_2022_program(program: &Address) {
    assert!(
        *program == Token2022::id(),
        "incorrect Token-2022 program id"
    );
}

fn address_to_pubkey(address: &Address) -> Pubkey {
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(address.as_ref());
    Pubkey::new_from_array(bytes)
}

fn optional_address_to_pubkey(address: Option<&Address>) -> Option<Pubkey> {
    address.map(address_to_pubkey)
}

fn build_data(instruction: Result<Instruction, ProgramError>) -> Vec<u8> {
    instruction
        .expect("failed to build Token-2022 extension instruction")
        .data
}

fn pubkey_refs(pubkeys: &[Pubkey]) -> Vec<&Pubkey> {
    pubkeys.iter().collect()
}

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
            InstructionAccount::new(self.authority.address(), false, false),
            InstructionAccount::readonly_signer(self.authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint, self.authority, self.authority]
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

pub struct DefaultAccountStateInitialize<'a> {
    pub mint: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for DefaultAccountStateInitialize<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.mint.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint]
    }
}

pub struct DefaultAccountStateUpdate<'a> {
    pub mint: CpiHandle<'a>,
    pub freeze_authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for DefaultAccountStateUpdate<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.mint.address()),
            InstructionAccount::readonly_signer(self.freeze_authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint, self.freeze_authority]
    }
}

pub struct ImmutableOwnerInitialize<'a> {
    pub token_account: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for ImmutableOwnerInitialize<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.token_account.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.token_account]
    }
}

pub struct InterestBearingMintInitialize<'a> {
    pub mint: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for InterestBearingMintInitialize<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.mint.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint]
    }
}

pub struct InterestBearingMintUpdateRate<'a> {
    pub mint: CpiHandle<'a>,
    pub rate_authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for InterestBearingMintUpdateRate<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.mint.address()),
            InstructionAccount::readonly_signer(self.rate_authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint, self.rate_authority]
    }
}

pub struct MemoTransfer<'a> {
    pub account: CpiHandle<'a>,
    pub owner: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for MemoTransfer<'a> {
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

pub struct MetadataPointerInitialize<'a> {
    pub mint: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for MetadataPointerInitialize<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.mint.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint]
    }
}

pub struct MetadataPointerUpdate<'a> {
    pub mint: CpiHandle<'a>,
    pub authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for MetadataPointerUpdate<'a> {
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

pub struct MintCloseAuthorityInitialize<'a> {
    pub mint: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for MintCloseAuthorityInitialize<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.mint.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint]
    }
}

pub struct NonTransferableMintInitialize<'a> {
    pub mint: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for NonTransferableMintInitialize<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.mint.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint]
    }
}

pub struct PermanentDelegateInitialize<'a> {
    pub mint: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for PermanentDelegateInitialize<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.mint.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint]
    }
}

pub struct TokenMetadataInitialize<'a> {
    pub metadata: CpiHandle<'a>,
    pub update_authority: CpiHandle<'a>,
    pub mint_authority: CpiHandle<'a>,
    pub mint: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for TokenMetadataInitialize<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.metadata.address()),
            InstructionAccount::new(self.update_authority.address(), false, false),
            InstructionAccount::new(self.mint.address(), false, false),
            InstructionAccount::readonly_signer(self.mint_authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![
            self.metadata,
            self.update_authority,
            self.mint,
            self.mint_authority,
        ]
    }
}

pub struct TokenMetadataUpdateAuthority<'a> {
    pub metadata: CpiHandle<'a>,
    pub current_authority: CpiHandle<'a>,
    pub new_authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for TokenMetadataUpdateAuthority<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.metadata.address()),
            InstructionAccount::readonly_signer(self.current_authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.metadata, self.current_authority]
    }
}

pub struct TokenMetadataUpdateField<'a> {
    pub metadata: CpiHandle<'a>,
    pub update_authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for TokenMetadataUpdateField<'a> {
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

pub struct TokenGroupInitialize<'a> {
    pub group: CpiHandle<'a>,
    pub mint: CpiHandle<'a>,
    pub mint_authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for TokenGroupInitialize<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.group.address()),
            InstructionAccount::new(self.mint.address(), false, false),
            InstructionAccount::readonly_signer(self.mint_authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.group, self.mint, self.mint_authority]
    }
}

pub struct TokenMemberInitialize<'a> {
    pub member: CpiHandle<'a>,
    pub member_mint: CpiHandle<'a>,
    pub member_mint_authority: CpiHandle<'a>,
    pub group: CpiHandle<'a>,
    pub group_update_authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for TokenMemberInitialize<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.member.address()),
            InstructionAccount::new(self.member_mint.address(), false, false),
            InstructionAccount::readonly_signer(self.member_mint_authority.address()),
            InstructionAccount::writable(self.group.address()),
            InstructionAccount::readonly_signer(self.group_update_authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![
            self.member,
            self.member_mint,
            self.member_mint_authority,
            self.group,
            self.group_update_authority,
        ]
    }
}

pub struct TransferFeeInitialize<'a> {
    pub mint: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for TransferFeeInitialize<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.mint.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint]
    }
}

pub struct TransferFeeSetTransferFee<'a> {
    pub mint: CpiHandle<'a>,
    pub authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for TransferFeeSetTransferFee<'a> {
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

pub struct TransferCheckedWithFee<'a> {
    pub source: CpiHandle<'a>,
    pub mint: CpiHandle<'a>,
    pub destination: CpiHandle<'a>,
    pub authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for TransferCheckedWithFee<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.source.address()),
            InstructionAccount::new(self.mint.address(), false, false),
            InstructionAccount::writable(self.destination.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.source, self.mint, self.destination, self.authority]
    }
}

pub struct HarvestWithheldTokensToMint<'a> {
    pub mint: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for HarvestWithheldTokensToMint<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.mint.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint]
    }
}

pub struct WithdrawWithheldTokensFromMint<'a> {
    pub mint: CpiHandle<'a>,
    pub destination: CpiHandle<'a>,
    pub authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for WithdrawWithheldTokensFromMint<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.mint.address()),
            InstructionAccount::writable(self.destination.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint, self.destination, self.authority]
    }
}

pub struct WithdrawWithheldTokensFromAccounts<'a> {
    pub mint: CpiHandle<'a>,
    pub destination: CpiHandle<'a>,
    pub authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for WithdrawWithheldTokensFromAccounts<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::new(self.mint.address(), false, false),
            InstructionAccount::writable(self.destination.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint, self.destination, self.authority]
    }
}

pub struct TransferHookInitialize<'a> {
    pub mint: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for TransferHookInitialize<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.mint.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint]
    }
}

pub struct TransferHookUpdate<'a> {
    pub mint: CpiHandle<'a>,
    pub authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for TransferHookUpdate<'a> {
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

fn encode_optional_address(address: Option<&Address>) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    if let Some(address) = address {
        assert!(
            address.as_ref().iter().any(|byte| *byte != 0),
            "optional address cannot be the zero address"
        );
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
    assert_token_2022_program(ctx.program);
    ctx.invoke(&[EXT_CPI_GUARD, DISC_INITIALIZE]);
}

#[deprecated(
    note = "Token-2022 rejects CPI-initiated toggling of CPI Guard with CpiGuardSettingsLocked."
)]
pub fn cpi_guard_disable<'a>(ctx: CpiContext<'a, CpiGuard<'a>>) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&[EXT_CPI_GUARD, DISC_UPDATE]);
}

pub fn group_pointer_initialize<'a>(
    ctx: CpiContext<'a, GroupPointerInitialize<'a>>,
    authority: Option<&Address>,
    group_address: Option<&Address>,
) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&encode_group_pointer_initialize(authority, group_address));
}

pub fn group_pointer_update<'a>(
    ctx: CpiContext<'a, GroupPointerUpdate<'a>>,
    group_address: Option<&Address>,
) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&encode_group_pointer_update(group_address));
}

pub fn group_member_pointer_initialize<'a>(
    ctx: CpiContext<'a, GroupMemberPointerInitialize<'a>>,
    authority: Option<&Address>,
    member_address: Option<&Address>,
) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&encode_group_member_pointer_initialize(
        authority,
        member_address,
    ));
}

pub fn group_member_pointer_update<'a>(
    ctx: CpiContext<'a, GroupMemberPointerUpdate<'a>>,
    member_address: Option<&Address>,
) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&encode_group_member_pointer_update(member_address));
}

pub fn pausable_initialize<'a>(ctx: CpiContext<'a, PausableInitialize<'a>>, authority: &Address) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&encode_pausable_initialize(authority));
}

pub fn pausable_pause<'a>(ctx: CpiContext<'a, PausableToggle<'a>>) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&[EXT_PAUSABLE, DISC_UPDATE]);
}

pub fn pausable_resume<'a>(ctx: CpiContext<'a, PausableToggle<'a>>) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&[EXT_PAUSABLE, DISC_RESUME]);
}

pub fn token_metadata_remove_key<'a>(
    ctx: CpiContext<'a, TokenMetadataRemoveKey<'a>>,
    key: String,
    idempotent: bool,
) {
    ctx.invoke(&encode_token_metadata_remove_key(&key, idempotent));
}

pub fn default_account_state_initialize<'a>(
    ctx: CpiContext<'a, DefaultAccountStateInitialize<'a>>,
    state: &spl_token_2022::state::AccountState,
) {
    assert_token_2022_program(ctx.program);
    let program = address_to_pubkey(ctx.program);
    let data = build_data(
        spl_token_2022::extension::default_account_state::instruction::initialize_default_account_state(
            &program,
            &address_to_pubkey(ctx.accounts.mint.address()),
            state,
        ),
    );
    ctx.invoke(&data);
}

pub fn default_account_state_update<'a>(
    ctx: CpiContext<'a, DefaultAccountStateUpdate<'a>>,
    state: &spl_token_2022::state::AccountState,
) {
    assert_token_2022_program(ctx.program);
    let program = address_to_pubkey(ctx.program);
    let data = build_data(
        spl_token_2022::extension::default_account_state::instruction::update_default_account_state(
            &program,
            &address_to_pubkey(ctx.accounts.mint.address()),
            &address_to_pubkey(ctx.accounts.freeze_authority.address()),
            &[],
            state,
        ),
    );
    ctx.invoke(&data);
}

pub fn immutable_owner_initialize<'a>(ctx: CpiContext<'a, ImmutableOwnerInitialize<'a>>) {
    assert_token_2022_program(ctx.program);
    let program = address_to_pubkey(ctx.program);
    let data = build_data(spl_token_2022::instruction::initialize_immutable_owner(
        &program,
        &address_to_pubkey(ctx.accounts.token_account.address()),
    ));
    ctx.invoke(&data);
}

pub fn interest_bearing_mint_initialize<'a>(
    ctx: CpiContext<'a, InterestBearingMintInitialize<'a>>,
    rate_authority: Option<&Address>,
    rate: i16,
) {
    assert_token_2022_program(ctx.program);
    let program = address_to_pubkey(ctx.program);
    let data = build_data(
        spl_token_2022::extension::interest_bearing_mint::instruction::initialize(
            &program,
            &address_to_pubkey(ctx.accounts.mint.address()),
            optional_address_to_pubkey(rate_authority),
            rate,
        ),
    );
    ctx.invoke(&data);
}

pub fn interest_bearing_mint_update_rate<'a>(
    ctx: CpiContext<'a, InterestBearingMintUpdateRate<'a>>,
    rate: i16,
) {
    assert_token_2022_program(ctx.program);
    let program = address_to_pubkey(ctx.program);
    let data = build_data(
        spl_token_2022::extension::interest_bearing_mint::instruction::update_rate(
            &program,
            &address_to_pubkey(ctx.accounts.mint.address()),
            &address_to_pubkey(ctx.accounts.rate_authority.address()),
            &[],
            rate,
        ),
    );
    ctx.invoke(&data);
}

pub fn memo_transfer_initialize<'a>(ctx: CpiContext<'a, MemoTransfer<'a>>) {
    assert_token_2022_program(ctx.program);
    let program = address_to_pubkey(ctx.program);
    let data = build_data(
        spl_token_2022::extension::memo_transfer::instruction::enable_required_transfer_memos(
            &program,
            &address_to_pubkey(ctx.accounts.account.address()),
            &address_to_pubkey(ctx.accounts.owner.address()),
            &[],
        ),
    );
    ctx.invoke(&data);
}

pub fn memo_transfer_disable<'a>(ctx: CpiContext<'a, MemoTransfer<'a>>) {
    assert_token_2022_program(ctx.program);
    let program = address_to_pubkey(ctx.program);
    let data = build_data(
        spl_token_2022::extension::memo_transfer::instruction::disable_required_transfer_memos(
            &program,
            &address_to_pubkey(ctx.accounts.account.address()),
            &address_to_pubkey(ctx.accounts.owner.address()),
            &[],
        ),
    );
    ctx.invoke(&data);
}

pub fn metadata_pointer_initialize<'a>(
    ctx: CpiContext<'a, MetadataPointerInitialize<'a>>,
    authority: Option<&Address>,
    metadata_address: Option<&Address>,
) {
    assert_token_2022_program(ctx.program);
    let program = address_to_pubkey(ctx.program);
    let data = build_data(
        spl_token_2022::extension::metadata_pointer::instruction::initialize(
            &program,
            &address_to_pubkey(ctx.accounts.mint.address()),
            optional_address_to_pubkey(authority),
            optional_address_to_pubkey(metadata_address),
        ),
    );
    ctx.invoke(&data);
}

pub fn metadata_pointer_update<'a>(
    ctx: CpiContext<'a, MetadataPointerUpdate<'a>>,
    metadata_address: Option<&Address>,
) {
    assert_token_2022_program(ctx.program);
    let program = address_to_pubkey(ctx.program);
    let data = build_data(
        spl_token_2022::extension::metadata_pointer::instruction::update(
            &program,
            &address_to_pubkey(ctx.accounts.mint.address()),
            &address_to_pubkey(ctx.accounts.authority.address()),
            &[],
            optional_address_to_pubkey(metadata_address),
        ),
    );
    ctx.invoke(&data);
}

pub fn mint_close_authority_initialize<'a>(
    ctx: CpiContext<'a, MintCloseAuthorityInitialize<'a>>,
    close_authority: Option<&Address>,
) {
    assert_token_2022_program(ctx.program);
    let program = address_to_pubkey(ctx.program);
    let close_authority = optional_address_to_pubkey(close_authority);
    let data = build_data(
        spl_token_2022::instruction::initialize_mint_close_authority(
            &program,
            &address_to_pubkey(ctx.accounts.mint.address()),
            close_authority.as_ref(),
        ),
    );
    ctx.invoke(&data);
}

pub fn non_transferable_mint_initialize<'a>(
    ctx: CpiContext<'a, NonTransferableMintInitialize<'a>>,
) {
    assert_token_2022_program(ctx.program);
    let program = address_to_pubkey(ctx.program);
    let data = build_data(
        spl_token_2022::instruction::initialize_non_transferable_mint(
            &program,
            &address_to_pubkey(ctx.accounts.mint.address()),
        ),
    );
    ctx.invoke(&data);
}

pub fn permanent_delegate_initialize<'a>(
    ctx: CpiContext<'a, PermanentDelegateInitialize<'a>>,
    permanent_delegate: &Address,
) {
    assert_token_2022_program(ctx.program);
    let program = address_to_pubkey(ctx.program);
    let data = build_data(spl_token_2022::instruction::initialize_permanent_delegate(
        &program,
        &address_to_pubkey(ctx.accounts.mint.address()),
        &address_to_pubkey(permanent_delegate),
    ));
    ctx.invoke(&data);
}

pub fn token_metadata_initialize<'a>(
    ctx: CpiContext<'a, TokenMetadataInitialize<'a>>,
    name: String,
    symbol: String,
    uri: String,
) {
    let program = address_to_pubkey(ctx.program);
    let ix = spl_token_metadata_interface::instruction::initialize(
        &program,
        &address_to_pubkey(ctx.accounts.metadata.address()),
        &address_to_pubkey(ctx.accounts.update_authority.address()),
        &address_to_pubkey(ctx.accounts.mint.address()),
        &address_to_pubkey(ctx.accounts.mint_authority.address()),
        name,
        symbol,
        uri,
    );
    ctx.invoke(&ix.data);
}

pub fn token_metadata_update_authority<'a>(
    ctx: CpiContext<'a, TokenMetadataUpdateAuthority<'a>>,
    new_authority: Option<&Address>,
) {
    let program = address_to_pubkey(ctx.program);
    let new_authority = OptionalNonZeroPubkey::try_from(optional_address_to_pubkey(new_authority))
        .expect("optional address cannot be the zero address");
    let ix = spl_token_metadata_interface::instruction::update_authority(
        &program,
        &address_to_pubkey(ctx.accounts.metadata.address()),
        &address_to_pubkey(ctx.accounts.current_authority.address()),
        new_authority,
    );
    ctx.invoke(&ix.data);
}

pub fn token_metadata_update_field<'a>(
    ctx: CpiContext<'a, TokenMetadataUpdateField<'a>>,
    field: Field,
    value: String,
) {
    let program = address_to_pubkey(ctx.program);
    let ix = spl_token_metadata_interface::instruction::update_field(
        &program,
        &address_to_pubkey(ctx.accounts.metadata.address()),
        &address_to_pubkey(ctx.accounts.update_authority.address()),
        field,
        value,
    );
    ctx.invoke(&ix.data);
}

pub fn token_group_initialize<'a>(
    ctx: CpiContext<'a, TokenGroupInitialize<'a>>,
    update_authority: Option<&Address>,
    max_size: u64,
) {
    let program = address_to_pubkey(ctx.program);
    let ix = spl_token_group_interface::instruction::initialize_group(
        &program,
        &address_to_pubkey(ctx.accounts.group.address()),
        &address_to_pubkey(ctx.accounts.mint.address()),
        &address_to_pubkey(ctx.accounts.mint_authority.address()),
        optional_address_to_pubkey(update_authority),
        max_size,
    );
    ctx.invoke(&ix.data);
}

pub fn token_member_initialize<'a>(ctx: CpiContext<'a, TokenMemberInitialize<'a>>) {
    let program = address_to_pubkey(ctx.program);
    let ix = spl_token_group_interface::instruction::initialize_member(
        &program,
        &address_to_pubkey(ctx.accounts.member.address()),
        &address_to_pubkey(ctx.accounts.member_mint.address()),
        &address_to_pubkey(ctx.accounts.member_mint_authority.address()),
        &address_to_pubkey(ctx.accounts.group.address()),
        &address_to_pubkey(ctx.accounts.group_update_authority.address()),
    );
    ctx.invoke(&ix.data);
}

pub fn transfer_fee_initialize<'a>(
    ctx: CpiContext<'a, TransferFeeInitialize<'a>>,
    transfer_fee_config_authority: Option<&Address>,
    withdraw_withheld_authority: Option<&Address>,
    transfer_fee_basis_points: u16,
    maximum_fee: u64,
) {
    assert_token_2022_program(ctx.program);
    let program = address_to_pubkey(ctx.program);
    let transfer_fee_config_authority = optional_address_to_pubkey(transfer_fee_config_authority);
    let withdraw_withheld_authority = optional_address_to_pubkey(withdraw_withheld_authority);
    let data = build_data(
        spl_token_2022::extension::transfer_fee::instruction::initialize_transfer_fee_config(
            &program,
            &address_to_pubkey(ctx.accounts.mint.address()),
            transfer_fee_config_authority.as_ref(),
            withdraw_withheld_authority.as_ref(),
            transfer_fee_basis_points,
            maximum_fee,
        ),
    );
    ctx.invoke(&data);
}

pub fn transfer_fee_set<'a>(
    ctx: CpiContext<'a, TransferFeeSetTransferFee<'a>>,
    transfer_fee_basis_points: u16,
    maximum_fee: u64,
) {
    assert_token_2022_program(ctx.program);
    let program = address_to_pubkey(ctx.program);
    let data = build_data(
        spl_token_2022::extension::transfer_fee::instruction::set_transfer_fee(
            &program,
            &address_to_pubkey(ctx.accounts.mint.address()),
            &address_to_pubkey(ctx.accounts.authority.address()),
            &[],
            transfer_fee_basis_points,
            maximum_fee,
        ),
    );
    ctx.invoke(&data);
}

pub fn transfer_checked_with_fee<'a>(
    ctx: CpiContext<'a, TransferCheckedWithFee<'a>>,
    amount: u64,
    decimals: u8,
    fee: u64,
) {
    assert_token_2022_program(ctx.program);
    let program = address_to_pubkey(ctx.program);
    let data = build_data(
        spl_token_2022::extension::transfer_fee::instruction::transfer_checked_with_fee(
            &program,
            &address_to_pubkey(ctx.accounts.source.address()),
            &address_to_pubkey(ctx.accounts.mint.address()),
            &address_to_pubkey(ctx.accounts.destination.address()),
            &address_to_pubkey(ctx.accounts.authority.address()),
            &[],
            amount,
            decimals,
            fee,
        ),
    );
    ctx.invoke(&data);
}

pub fn harvest_withheld_tokens_to_mint<'a>(
    ctx: CpiContext<'a, HarvestWithheldTokensToMint<'a>>,
    sources: Vec<CpiHandle<'a>>,
) {
    assert_token_2022_program(ctx.program);
    let program = address_to_pubkey(ctx.program);
    let source_pubkeys: Vec<Pubkey> = sources
        .iter()
        .map(|source| address_to_pubkey(source.address()))
        .collect();
    let source_refs = pubkey_refs(&source_pubkeys);
    let data = build_data(
        spl_token_2022::extension::transfer_fee::instruction::harvest_withheld_tokens_to_mint(
            &program,
            &address_to_pubkey(ctx.accounts.mint.address()),
            &source_refs,
        ),
    );
    ctx.with_remaining_accounts(sources).invoke(&data);
}

pub fn withdraw_withheld_tokens_from_mint<'a>(
    ctx: CpiContext<'a, WithdrawWithheldTokensFromMint<'a>>,
) {
    assert_token_2022_program(ctx.program);
    let program = address_to_pubkey(ctx.program);
    let data = build_data(
        spl_token_2022::extension::transfer_fee::instruction::withdraw_withheld_tokens_from_mint(
            &program,
            &address_to_pubkey(ctx.accounts.mint.address()),
            &address_to_pubkey(ctx.accounts.destination.address()),
            &address_to_pubkey(ctx.accounts.authority.address()),
            &[],
        ),
    );
    ctx.invoke(&data);
}

pub fn withdraw_withheld_tokens_from_accounts<'a>(
    ctx: CpiContext<'a, WithdrawWithheldTokensFromAccounts<'a>>,
    sources: Vec<CpiHandle<'a>>,
) {
    assert_token_2022_program(ctx.program);
    let program = address_to_pubkey(ctx.program);
    let source_pubkeys: Vec<Pubkey> = sources
        .iter()
        .map(|source| address_to_pubkey(source.address()))
        .collect();
    let source_refs = pubkey_refs(&source_pubkeys);
    let data = build_data(
        spl_token_2022::extension::transfer_fee::instruction::withdraw_withheld_tokens_from_accounts(
            &program,
            &address_to_pubkey(ctx.accounts.mint.address()),
            &address_to_pubkey(ctx.accounts.destination.address()),
            &address_to_pubkey(ctx.accounts.authority.address()),
            &[],
            &source_refs,
        ),
    );
    ctx.with_remaining_accounts(sources).invoke(&data);
}

pub fn transfer_hook_initialize<'a>(
    ctx: CpiContext<'a, TransferHookInitialize<'a>>,
    authority: Option<&Address>,
    transfer_hook_program_id: Option<&Address>,
) {
    assert_token_2022_program(ctx.program);
    let program = address_to_pubkey(ctx.program);
    let data = build_data(
        spl_token_2022::extension::transfer_hook::instruction::initialize(
            &program,
            &address_to_pubkey(ctx.accounts.mint.address()),
            optional_address_to_pubkey(authority),
            optional_address_to_pubkey(transfer_hook_program_id),
        ),
    );
    ctx.invoke(&data);
}

pub fn transfer_hook_update<'a>(
    ctx: CpiContext<'a, TransferHookUpdate<'a>>,
    transfer_hook_program_id: Option<&Address>,
) {
    assert_token_2022_program(ctx.program);
    let program = address_to_pubkey(ctx.program);
    let data = build_data(
        spl_token_2022::extension::transfer_hook::instruction::update(
            &program,
            &address_to_pubkey(ctx.accounts.mint.address()),
            &address_to_pubkey(ctx.accounts.authority.address()),
            &[],
            optional_address_to_pubkey(transfer_hook_program_id),
        ),
    );
    ctx.invoke(&data);
}

#[allow(deprecated)]
pub mod cpi_guard {
    pub use super::{cpi_guard_disable, cpi_guard_enable, CpiGuard};
}

pub mod default_account_state {
    pub use super::{
        default_account_state_initialize, default_account_state_update,
        DefaultAccountStateInitialize, DefaultAccountStateUpdate,
    };
}

pub mod group_pointer {
    pub use super::{
        group_pointer_initialize, group_pointer_update, GroupPointerInitialize, GroupPointerUpdate,
    };
}

pub mod group_member_pointer {
    pub use super::{
        group_member_pointer_initialize, group_member_pointer_update, GroupMemberPointerInitialize,
        GroupMemberPointerUpdate,
    };
}

pub mod immutable_owner {
    pub use super::{immutable_owner_initialize, ImmutableOwnerInitialize};
}

pub mod interest_bearing_mint {
    pub use super::{
        interest_bearing_mint_initialize, interest_bearing_mint_update_rate,
        InterestBearingMintInitialize, InterestBearingMintUpdateRate,
    };
}

pub mod memo_transfer {
    pub use super::{memo_transfer_disable, memo_transfer_initialize, MemoTransfer};
}

pub mod metadata_pointer {
    pub use super::{
        metadata_pointer_initialize, metadata_pointer_update, MetadataPointerInitialize,
        MetadataPointerUpdate,
    };
}

pub mod mint_close_authority {
    pub use super::{mint_close_authority_initialize, MintCloseAuthorityInitialize};
}

pub mod non_transferable {
    pub use super::{non_transferable_mint_initialize, NonTransferableMintInitialize};
}

pub mod pausable {
    pub use super::{
        pausable_initialize, pausable_pause, pausable_resume, PausableInitialize, PausableToggle,
    };
}

pub mod permanent_delegate {
    pub use super::{permanent_delegate_initialize, PermanentDelegateInitialize};
}

pub mod token_group {
    pub use super::{
        token_group_initialize, token_member_initialize, TokenGroupInitialize,
        TokenMemberInitialize,
    };
}

pub mod token_metadata {
    pub use super::{
        token_metadata_initialize, token_metadata_remove_key, token_metadata_update_authority,
        token_metadata_update_field, TokenMetadataInitialize, TokenMetadataRemoveKey,
        TokenMetadataUpdateAuthority, TokenMetadataUpdateField,
    };
    pub use spl_token_metadata_interface::state::Field;
}

pub mod transfer_fee {
    pub use super::{
        harvest_withheld_tokens_to_mint, transfer_checked_with_fee, transfer_fee_initialize,
        transfer_fee_set, withdraw_withheld_tokens_from_accounts,
        withdraw_withheld_tokens_from_mint, HarvestWithheldTokensToMint, TransferCheckedWithFee,
        TransferFeeInitialize, TransferFeeSetTransferFee, WithdrawWithheldTokensFromAccounts,
        WithdrawWithheldTokensFromMint,
    };
}

pub mod transfer_hook {
    pub use super::{
        transfer_hook_initialize, transfer_hook_update, TransferHookInitialize, TransferHookUpdate,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_2022_program_check_accepts_canonical_id() {
        assert_token_2022_program(&Token2022::id());
    }

    #[test]
    #[should_panic(expected = "incorrect Token-2022 program id")]
    fn token_2022_program_check_rejects_other_programs() {
        assert_token_2022_program(&Address::new_from_array([1; 32]));
    }

    #[test]
    fn optional_address_encodes_none_and_nonzero_some() {
        let address = Address::new_from_array([7; 32]);

        assert_eq!(encode_optional_address(None), [0; 32]);
        assert_eq!(encode_optional_address(Some(&address)), [7; 32]);
    }

    #[test]
    #[should_panic(expected = "optional address cannot be the zero address")]
    fn optional_address_rejects_zero_some() {
        let zero = Address::new_from_array([0; 32]);

        let _ = encode_optional_address(Some(&zero));
    }

    #[test]
    fn group_pointer_update_encoder_matches_v1_layout() {
        let group = Address::new_from_array([9; 32]);
        let mut expected = [0; 34];
        expected[0] = EXT_GROUP_POINTER;
        expected[1] = DISC_UPDATE;
        expected[2..34].copy_from_slice(group.as_ref());

        assert_eq!(encode_group_pointer_update(Some(&group)), expected);
    }

    #[test]
    fn token_metadata_remove_key_encoder_matches_interface_discriminator() {
        let data = encode_token_metadata_remove_key("field", true);

        assert_eq!(&data[..8], &TOKEN_METADATA_REMOVE_KEY_DISCRIMINATOR);
        assert_eq!(data[8], 1);
        assert_eq!(&data[9..13], &5u32.to_le_bytes());
        assert_eq!(&data[13..], b"field");
    }
}
