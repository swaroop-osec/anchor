use {
    super::common::{pubkey_refs, validate_token_2022_program},
    crate::token_2022::spl_token_2022,
    alloc::{vec, vec::Vec},
    anchor_lang_v2::{CpiContext, CpiHandle, ToCpiAccounts},
    pinocchio::{address::Address, instruction::InstructionAccount},
    solana_program_error::ProgramError,
    solana_pubkey::Pubkey,
};

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

pub fn transfer_fee_initialize<'a>(
    ctx: CpiContext<'a, TransferFeeInitialize<'a>>,
    transfer_fee_config_authority: Option<&Address>,
    withdraw_withheld_authority: Option<&Address>,
    transfer_fee_basis_points: u16,
    maximum_fee: u64,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let program = *ctx.program;
    let transfer_fee_config_authority = transfer_fee_config_authority.copied();
    let withdraw_withheld_authority = withdraw_withheld_authority.copied();
    let ix = spl_token_2022::extension::transfer_fee::instruction::initialize_transfer_fee_config(
        &program,
        ctx.accounts.mint.address(),
        transfer_fee_config_authority.as_ref(),
        withdraw_withheld_authority.as_ref(),
        transfer_fee_basis_points,
        maximum_fee,
    )?;
    ctx.invoke_ix(ix)
}

pub fn transfer_fee_set<'a>(
    ctx: CpiContext<'a, TransferFeeSetTransferFee<'a>>,
    transfer_fee_basis_points: u16,
    maximum_fee: u64,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let program = *ctx.program;
    let ix = spl_token_2022::extension::transfer_fee::instruction::set_transfer_fee(
        &program,
        ctx.accounts.mint.address(),
        ctx.accounts.authority.address(),
        &[],
        transfer_fee_basis_points,
        maximum_fee,
    )?;
    ctx.invoke_ix(ix)
}

pub fn transfer_checked_with_fee<'a>(
    ctx: CpiContext<'a, TransferCheckedWithFee<'a>>,
    amount: u64,
    decimals: u8,
    fee: u64,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let program = *ctx.program;
    let ix = spl_token_2022::extension::transfer_fee::instruction::transfer_checked_with_fee(
        &program,
        ctx.accounts.source.address(),
        ctx.accounts.mint.address(),
        ctx.accounts.destination.address(),
        ctx.accounts.authority.address(),
        &[],
        amount,
        decimals,
        fee,
    )?;
    ctx.invoke_ix(ix)
}

pub fn harvest_withheld_tokens_to_mint<'a>(
    ctx: CpiContext<'a, HarvestWithheldTokensToMint<'a>>,
    sources: Vec<CpiHandle<'a>>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let program = *ctx.program;
    let source_pubkeys: Vec<Pubkey> = sources.iter().map(|source| *source.address()).collect();
    let source_refs = pubkey_refs(&source_pubkeys);
    let ix = spl_token_2022::extension::transfer_fee::instruction::harvest_withheld_tokens_to_mint(
        &program,
        ctx.accounts.mint.address(),
        &source_refs,
    )?;
    let mut remaining_accounts = sources;
    remaining_accounts.extend(ctx.remaining_accounts.iter().copied());
    ctx.with_remaining_accounts(remaining_accounts)
        .invoke_ix(ix)
}

pub fn withdraw_withheld_tokens_from_mint<'a>(
    ctx: CpiContext<'a, WithdrawWithheldTokensFromMint<'a>>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let program = *ctx.program;
    let ix =
        spl_token_2022::extension::transfer_fee::instruction::withdraw_withheld_tokens_from_mint(
            &program,
            ctx.accounts.mint.address(),
            ctx.accounts.destination.address(),
            ctx.accounts.authority.address(),
            &[],
        )?;
    ctx.invoke_ix(ix)
}

pub fn withdraw_withheld_tokens_from_accounts<'a>(
    ctx: CpiContext<'a, WithdrawWithheldTokensFromAccounts<'a>>,
    sources: Vec<CpiHandle<'a>>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let program = *ctx.program;
    let source_pubkeys: Vec<Pubkey> = sources.iter().map(|source| *source.address()).collect();
    let source_refs = pubkey_refs(&source_pubkeys);
    let ix =
        spl_token_2022::extension::transfer_fee::instruction::withdraw_withheld_tokens_from_accounts(
            &program,
            ctx.accounts.mint.address(),
            ctx.accounts.destination.address(),
            ctx.accounts.authority.address(),
            &[],
            &source_refs,
        )?;
    let mut remaining_accounts = sources;
    remaining_accounts.extend(ctx.remaining_accounts.iter().copied());
    ctx.with_remaining_accounts(remaining_accounts)
        .invoke_ix(ix)
}
