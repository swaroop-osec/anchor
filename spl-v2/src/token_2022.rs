//! Token-2022 CPI helpers and interface re-exports.

extern crate alloc;

use {
    alloc::{string::String, vec, vec::Vec},
    anchor_lang_v2::{CpiContext, CpiHandle, Id, ToCpiAccounts},
    pinocchio::{address::Address, instruction::InstructionAccount},
    solana_program_error::ProgramError,
    solana_pubkey::Pubkey,
};

pub use anchor_lang_v2::programs::Token2022;
pub use spl_token_2022_interface::{self as spl_token_2022, extension::ExtensionType, ID};

pub struct Transfer<'a> {
    pub from: CpiHandle<'a>,
    pub to: CpiHandle<'a>,
    pub authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for Transfer<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.from.address()),
            InstructionAccount::writable(self.to.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.from, self.to, self.authority]
    }
}

pub struct TransferChecked<'a> {
    pub from: CpiHandle<'a>,
    pub mint: CpiHandle<'a>,
    pub to: CpiHandle<'a>,
    pub authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for TransferChecked<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.from.address()),
            InstructionAccount::new(self.mint.address(), false, false),
            InstructionAccount::writable(self.to.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.from, self.mint, self.to, self.authority]
    }
}

pub struct MintTo<'a> {
    pub mint: CpiHandle<'a>,
    pub to: CpiHandle<'a>,
    pub authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for MintTo<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.mint.address()),
            InstructionAccount::writable(self.to.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint, self.to, self.authority]
    }
}

pub struct MintToChecked<'a> {
    pub mint: CpiHandle<'a>,
    pub to: CpiHandle<'a>,
    pub authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for MintToChecked<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.mint.address()),
            InstructionAccount::writable(self.to.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint, self.to, self.authority]
    }
}

pub struct Burn<'a> {
    pub mint: CpiHandle<'a>,
    pub from: CpiHandle<'a>,
    pub authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for Burn<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.from.address()),
            InstructionAccount::writable(self.mint.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.from, self.mint, self.authority]
    }
}

pub struct BurnChecked<'a> {
    pub mint: CpiHandle<'a>,
    pub from: CpiHandle<'a>,
    pub authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for BurnChecked<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.from.address()),
            InstructionAccount::writable(self.mint.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.from, self.mint, self.authority]
    }
}

pub struct Approve<'a> {
    pub to: CpiHandle<'a>,
    pub delegate: CpiHandle<'a>,
    pub authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for Approve<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.to.address()),
            InstructionAccount::new(self.delegate.address(), false, false),
            InstructionAccount::readonly_signer(self.authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.to, self.delegate, self.authority]
    }
}

pub struct ApproveChecked<'a> {
    pub to: CpiHandle<'a>,
    pub mint: CpiHandle<'a>,
    pub delegate: CpiHandle<'a>,
    pub authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for ApproveChecked<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.to.address()),
            InstructionAccount::new(self.mint.address(), false, false),
            InstructionAccount::new(self.delegate.address(), false, false),
            InstructionAccount::readonly_signer(self.authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.to, self.mint, self.delegate, self.authority]
    }
}

pub struct Revoke<'a> {
    pub source: CpiHandle<'a>,
    pub authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for Revoke<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.source.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.source, self.authority]
    }
}

pub struct InitializeAccount<'a> {
    pub account: CpiHandle<'a>,
    pub mint: CpiHandle<'a>,
    pub authority: CpiHandle<'a>,
    pub rent: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for InitializeAccount<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.account.address()),
            InstructionAccount::new(self.mint.address(), false, false),
            InstructionAccount::new(self.authority.address(), false, false),
            InstructionAccount::new(self.rent.address(), false, false),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.account, self.mint, self.authority, self.rent]
    }
}

pub struct InitializeAccount3<'a> {
    pub account: CpiHandle<'a>,
    pub mint: CpiHandle<'a>,
    pub authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for InitializeAccount3<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.account.address()),
            InstructionAccount::new(self.mint.address(), false, false),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.account, self.mint]
    }
}

pub struct CloseAccount<'a> {
    pub account: CpiHandle<'a>,
    pub destination: CpiHandle<'a>,
    pub authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for CloseAccount<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.account.address()),
            InstructionAccount::writable(self.destination.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.account, self.destination, self.authority]
    }
}

pub struct FreezeAccount<'a> {
    pub account: CpiHandle<'a>,
    pub mint: CpiHandle<'a>,
    pub authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for FreezeAccount<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.account.address()),
            InstructionAccount::new(self.mint.address(), false, false),
            InstructionAccount::readonly_signer(self.authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.account, self.mint, self.authority]
    }
}

pub struct ThawAccount<'a> {
    pub account: CpiHandle<'a>,
    pub mint: CpiHandle<'a>,
    pub authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for ThawAccount<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.account.address()),
            InstructionAccount::new(self.mint.address(), false, false),
            InstructionAccount::readonly_signer(self.authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.account, self.mint, self.authority]
    }
}

pub struct InitializeMint<'a> {
    pub mint: CpiHandle<'a>,
    pub rent: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for InitializeMint<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.mint.address()),
            InstructionAccount::new(self.rent.address(), false, false),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint, self.rent]
    }
}

pub struct InitializeMint2<'a> {
    pub mint: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for InitializeMint2<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.mint.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint]
    }
}

pub struct SetAuthority<'a> {
    pub current_authority: CpiHandle<'a>,
    pub account_or_mint: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for SetAuthority<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.account_or_mint.address()),
            InstructionAccount::readonly_signer(self.current_authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.account_or_mint, self.current_authority]
    }
}

pub struct SyncNative<'a> {
    pub account: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for SyncNative<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.account.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.account]
    }
}

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
    pub mint: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for InitializeMintCloseAuthority<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.mint.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint]
    }
}

pub struct InitializeImmutableOwner<'a> {
    pub account: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for InitializeImmutableOwner<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.account.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.account]
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

const DISC_INITIALIZE_MINT: u8 = 0;
const DISC_INITIALIZE_ACCOUNT: u8 = 1;
const DISC_TRANSFER: u8 = 3;
const DISC_APPROVE: u8 = 4;
const DISC_REVOKE: u8 = 5;
const DISC_MINT_TO: u8 = 7;
const DISC_BURN: u8 = 8;
const DISC_CLOSE_ACCOUNT: u8 = 9;
const DISC_FREEZE_ACCOUNT: u8 = 10;
const DISC_THAW_ACCOUNT: u8 = 11;
const DISC_TRANSFER_CHECKED: u8 = 12;
const DISC_APPROVE_CHECKED: u8 = 13;
const DISC_MINT_TO_CHECKED: u8 = 14;
const DISC_BURN_CHECKED: u8 = 15;
const DISC_SYNC_NATIVE: u8 = 17;
const DISC_INITIALIZE_MINT2: u8 = 20;
const DISC_GET_ACCOUNT_DATA_SIZE: u8 = 21;
const DISC_INITIALIZE_IMMUTABLE_OWNER: u8 = 22;
const DISC_AMOUNT_TO_UI_AMOUNT: u8 = 23;
const DISC_UI_AMOUNT_TO_AMOUNT: u8 = 24;
const DISC_INITIALIZE_MINT_CLOSE_AUTHORITY: u8 = 25;
const DISC_INITIALIZE_PERMANENT_DELEGATE: u8 = 35;
const DISC_REALLOCATE: u8 = 29;
const DISC_CREATE_NATIVE_MINT: u8 = 31;
const DISC_INITIALIZE_NON_TRANSFERABLE_MINT: u8 = 32;
const DISC_WITHDRAW_EXCESS_LAMPORTS: u8 = 38;

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

#[inline]
fn encode_amount_ix(disc: u8, amount: u64) -> [u8; 9] {
    let mut data = [0u8; 9];
    data[0] = disc;
    data[1..9].copy_from_slice(&amount.to_le_bytes());
    data
}

#[inline]
fn encode_amount_decimals_ix(disc: u8, amount: u64, decimals: u8) -> [u8; 10] {
    let mut data = [0u8; 10];
    data[0] = disc;
    data[1..9].copy_from_slice(&amount.to_le_bytes());
    data[9] = decimals;
    data
}

fn encode_address_option_ix(disc: u8, address: Option<&Address>) -> Vec<u8> {
    let mut data = Vec::with_capacity(34);
    data.push(disc);
    match address {
        Some(address) => {
            data.push(1);
            data.extend_from_slice(address.as_ref());
        }
        None => data.push(0),
    }
    data
}

fn encode_initialize_mint_ix(
    disc: u8,
    decimals: u8,
    authority: &Address,
    freeze_authority: Option<&Address>,
) -> Vec<u8> {
    let mut data = Vec::with_capacity(67);
    data.push(disc);
    data.push(decimals);
    data.extend_from_slice(authority.as_ref());
    match freeze_authority {
        Some(authority) => {
            data.push(1);
            data.extend_from_slice(authority.as_ref());
        }
        None => data.push(0),
    }
    data
}

fn encode_permanent_delegate_ix(delegate: &Address) -> [u8; 33] {
    let mut data = [0u8; 33];
    data[0] = DISC_INITIALIZE_PERMANENT_DELEGATE;
    data[1..33].copy_from_slice(delegate.as_ref());
    data
}

fn encode_reallocate_ix(extension_types: &[ExtensionType]) -> Vec<u8> {
    let mut data = Vec::with_capacity(1 + extension_types.len() * 2);
    data.push(DISC_REALLOCATE);
    for extension_type in extension_types {
        data.extend_from_slice(&<[u8; 2]>::from(*extension_type));
    }
    data
}

fn encode_get_account_data_size_ix(extension_types: &[ExtensionType]) -> Vec<u8> {
    let mut data = Vec::with_capacity(1 + extension_types.len() * 2);
    data.push(DISC_GET_ACCOUNT_DATA_SIZE);
    for extension_type in extension_types {
        data.extend_from_slice(&<[u8; 2]>::from(*extension_type));
    }
    data
}

fn encode_ui_amount_to_amount_ix(ui_amount: &str) -> Vec<u8> {
    let bytes = ui_amount.as_bytes();
    let mut data = Vec::with_capacity(1 + bytes.len());
    data.push(DISC_UI_AMOUNT_TO_AMOUNT);
    data.extend_from_slice(bytes);
    data
}

fn build_data<T>(instruction: Result<T, ProgramError>) -> Vec<u8>
where
    T: IntoInstructionData,
{
    instruction
        .expect("failed to build Token-2022 instruction")
        .into_data()
}

trait IntoInstructionData {
    fn into_data(self) -> Vec<u8>;
}

impl IntoInstructionData for solana_instruction::Instruction {
    fn into_data(self) -> Vec<u8> {
        self.data
    }
}

fn return_data_from(program: &Address) -> Vec<u8> {
    let (return_program, data) = solana_cpi::get_return_data().expect("missing return data");
    assert_eq!(
        return_program.to_bytes().as_slice(),
        program.as_ref(),
        "return data from incorrect program"
    );
    data
}

pub fn transfer<'a>(ctx: CpiContext<'a, Transfer<'a>>, amount: u64) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&encode_amount_ix(DISC_TRANSFER, amount));
}

pub fn transfer_checked<'a>(ctx: CpiContext<'a, TransferChecked<'a>>, amount: u64, decimals: u8) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&encode_amount_decimals_ix(
        DISC_TRANSFER_CHECKED,
        amount,
        decimals,
    ));
}

pub fn mint_to<'a>(ctx: CpiContext<'a, MintTo<'a>>, amount: u64) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&encode_amount_ix(DISC_MINT_TO, amount));
}

pub fn mint_to_checked<'a>(ctx: CpiContext<'a, MintToChecked<'a>>, amount: u64, decimals: u8) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&encode_amount_decimals_ix(
        DISC_MINT_TO_CHECKED,
        amount,
        decimals,
    ));
}

pub fn burn<'a>(ctx: CpiContext<'a, Burn<'a>>, amount: u64) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&encode_amount_ix(DISC_BURN, amount));
}

pub fn burn_checked<'a>(ctx: CpiContext<'a, BurnChecked<'a>>, amount: u64, decimals: u8) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&encode_amount_decimals_ix(
        DISC_BURN_CHECKED,
        amount,
        decimals,
    ));
}

pub fn approve<'a>(ctx: CpiContext<'a, Approve<'a>>, amount: u64) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&encode_amount_ix(DISC_APPROVE, amount));
}

pub fn approve_checked<'a>(ctx: CpiContext<'a, ApproveChecked<'a>>, amount: u64, decimals: u8) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&encode_amount_decimals_ix(
        DISC_APPROVE_CHECKED,
        amount,
        decimals,
    ));
}

pub fn revoke<'a>(ctx: CpiContext<'a, Revoke<'a>>) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&[DISC_REVOKE]);
}

pub fn initialize_account<'a>(ctx: CpiContext<'a, InitializeAccount<'a>>) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&[DISC_INITIALIZE_ACCOUNT]);
}

pub fn initialize_account3<'a>(ctx: CpiContext<'a, InitializeAccount3<'a>>) {
    assert_token_2022_program(ctx.program);
    let owner = address_to_pubkey(ctx.accounts.authority.address());
    ctx.invoke(&build_data(
        spl_token_2022::instruction::initialize_account3(
            &address_to_pubkey(ctx.program),
            &address_to_pubkey(ctx.accounts.account.address()),
            &address_to_pubkey(ctx.accounts.mint.address()),
            &owner,
        ),
    ));
}

pub fn close_account<'a>(ctx: CpiContext<'a, CloseAccount<'a>>) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&[DISC_CLOSE_ACCOUNT]);
}

pub fn freeze_account<'a>(ctx: CpiContext<'a, FreezeAccount<'a>>) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&[DISC_FREEZE_ACCOUNT]);
}

pub fn thaw_account<'a>(ctx: CpiContext<'a, ThawAccount<'a>>) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&[DISC_THAW_ACCOUNT]);
}

pub fn initialize_mint<'a>(
    ctx: CpiContext<'a, InitializeMint<'a>>,
    decimals: u8,
    authority: &Address,
    freeze_authority: Option<&Address>,
) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&encode_initialize_mint_ix(
        DISC_INITIALIZE_MINT,
        decimals,
        authority,
        freeze_authority,
    ));
}

pub fn initialize_mint2<'a>(
    ctx: CpiContext<'a, InitializeMint2<'a>>,
    decimals: u8,
    authority: &Address,
    freeze_authority: Option<&Address>,
) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&encode_initialize_mint_ix(
        DISC_INITIALIZE_MINT2,
        decimals,
        authority,
        freeze_authority,
    ));
}

pub fn set_authority<'a>(
    ctx: CpiContext<'a, SetAuthority<'a>>,
    authority_type: spl_token_2022::instruction::AuthorityType,
    new_authority: Option<&Address>,
) {
    assert_token_2022_program(ctx.program);
    let data = spl_token_2022::instruction::set_authority(
        &address_to_pubkey(ctx.program),
        &address_to_pubkey(ctx.accounts.account_or_mint.address()),
        optional_address_to_pubkey(new_authority).as_ref(),
        authority_type,
        &address_to_pubkey(ctx.accounts.current_authority.address()),
        &[],
    )
    .expect("failed to build Token-2022 set_authority instruction")
    .data;
    ctx.invoke(&data);
}

pub fn sync_native<'a>(ctx: CpiContext<'a, SyncNative<'a>>) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&[DISC_SYNC_NATIVE]);
}

pub fn get_account_data_size<'a>(
    ctx: CpiContext<'a, GetAccountDataSize<'a>>,
    extension_types: &[ExtensionType],
) -> u64 {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&encode_get_account_data_size_ix(extension_types));
    let data = return_data_from(ctx.program);
    let bytes: [u8; 8] = data
        .as_slice()
        .try_into()
        .expect("invalid get_account_data_size return data");
    u64::from_le_bytes(bytes)
}

pub fn initialize_mint_close_authority<'a>(
    ctx: CpiContext<'a, InitializeMintCloseAuthority<'a>>,
    close_authority: Option<&Address>,
) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&encode_address_option_ix(
        DISC_INITIALIZE_MINT_CLOSE_AUTHORITY,
        close_authority,
    ));
}

pub fn initialize_immutable_owner<'a>(ctx: CpiContext<'a, InitializeImmutableOwner<'a>>) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&[DISC_INITIALIZE_IMMUTABLE_OWNER]);
}

pub fn amount_to_ui_amount<'a>(ctx: CpiContext<'a, AmountToUiAmount<'a>>, amount: u64) -> String {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&encode_amount_ix(DISC_AMOUNT_TO_UI_AMOUNT, amount));
    String::from_utf8(return_data_from(ctx.program))
        .expect("invalid amount_to_ui_amount return data")
}

pub fn ui_amount_to_amount<'a>(ctx: CpiContext<'a, UiAmountToAmount<'a>>, ui_amount: &str) -> u64 {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&encode_ui_amount_to_amount_ix(ui_amount));
    let data = return_data_from(ctx.program);
    let bytes: [u8; 8] = data
        .as_slice()
        .try_into()
        .expect("invalid ui_amount_to_amount return data");
    u64::from_le_bytes(bytes)
}

pub fn reallocate<'a>(ctx: CpiContext<'a, Reallocate<'a>>, extension_types: &[ExtensionType]) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&encode_reallocate_ix(extension_types));
}

pub fn withdraw_excess_lamports<'a>(ctx: CpiContext<'a, WithdrawExcessLamports<'a>>) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&[DISC_WITHDRAW_EXCESS_LAMPORTS]);
}

pub fn create_native_mint<'a>(ctx: CpiContext<'a, CreateNativeMint<'a>>) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&[DISC_CREATE_NATIVE_MINT]);
}

pub fn initialize_non_transferable_mint<'a>(
    ctx: CpiContext<'a, InitializeNonTransferableMint<'a>>,
) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&[DISC_INITIALIZE_NON_TRANSFERABLE_MINT]);
}

pub fn initialize_permanent_delegate<'a>(
    ctx: CpiContext<'a, PermanentDelegateInitialize<'a>>,
    permanent_delegate: &Address,
) {
    assert_token_2022_program(ctx.program);
    ctx.invoke(&encode_permanent_delegate_ix(permanent_delegate));
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
    fn reallocate_encoder_matches_v1_extension_discriminants() {
        assert_eq!(
            encode_reallocate_ix(&[ExtensionType::GroupPointer, ExtensionType::PausableAccount]),
            vec![DISC_REALLOCATE, 20, 0, 27, 0]
        );
    }

    #[test]
    fn initialize_mint_encoder_uses_coption_layout() {
        let authority = Address::new_from_array([7; 32]);
        let freeze_authority = Address::new_from_array([8; 32]);
        let data =
            encode_initialize_mint_ix(DISC_INITIALIZE_MINT, 9, &authority, Some(&freeze_authority));
        let expected = spl_token_2022::instruction::initialize_mint(
            &address_to_pubkey(&Token2022::id()),
            &Pubkey::new_unique(),
            &address_to_pubkey(&authority),
            Some(&address_to_pubkey(&freeze_authority)),
            9,
        )
        .unwrap()
        .data;

        assert_eq!(data[0], DISC_INITIALIZE_MINT);
        assert_eq!(data[1], 9);
        assert_eq!(&data[2..34], authority.as_ref());
        assert_eq!(data[34], 1);
        assert_eq!(&data[35..67], freeze_authority.as_ref());
        assert_eq!(data, expected);
        assert_eq!(
            encode_initialize_mint_ix(DISC_INITIALIZE_MINT, 9, &authority, None),
            spl_token_2022::instruction::initialize_mint(
                &address_to_pubkey(&Token2022::id()),
                &Pubkey::new_unique(),
                &address_to_pubkey(&authority),
                None,
                9,
            )
            .unwrap()
            .data
        );
    }

    #[test]
    fn option_and_string_encoders_match_interface_layout() {
        let address = Address::new_from_array([9; 32]);

        assert_eq!(
            encode_address_option_ix(DISC_INITIALIZE_MINT_CLOSE_AUTHORITY, Some(&address)),
            spl_token_2022::instruction::initialize_mint_close_authority(
                &address_to_pubkey(&Token2022::id()),
                &Pubkey::new_unique(),
                Some(&address_to_pubkey(&address)),
            )
            .unwrap()
            .data
        );
        assert_eq!(
            encode_address_option_ix(DISC_INITIALIZE_MINT_CLOSE_AUTHORITY, None),
            spl_token_2022::instruction::initialize_mint_close_authority(
                &address_to_pubkey(&Token2022::id()),
                &Pubkey::new_unique(),
                None,
            )
            .unwrap()
            .data
        );
        assert_eq!(
            encode_ui_amount_to_amount_ix("1.25"),
            spl_token_2022::instruction::ui_amount_to_amount(
                &address_to_pubkey(&Token2022::id()),
                &Pubkey::new_unique(),
                "1.25",
            )
            .unwrap()
            .data
        );
    }
}
