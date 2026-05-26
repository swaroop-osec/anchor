//! Shared base token CPI helpers used by `token` and `token_2022`.

extern crate alloc;

use {
    alloc::{vec, vec::Vec},
    anchor_lang_v2::{CpiContext, CpiHandle, Id, ToCpiAccounts},
    pinocchio::{address::Address, instruction::InstructionAccount},
    solana_program_error::ProgramError,
    spl_token_2022_interface as spl_token_2022,
};

#[cfg(feature = "guardrails")]
#[inline]
pub(crate) fn validate_token_interface_program(program_id: &Address) -> Result<(), ProgramError> {
    if anchor_lang_v2::address_eq(program_id, &anchor_lang_v2::programs::Token::id())
        || anchor_lang_v2::address_eq(program_id, &anchor_lang_v2::programs::Token2022::id())
    {
        Ok(())
    } else {
        Err(ProgramError::IncorrectProgramId)
    }
}

#[cfg(not(feature = "guardrails"))]
#[inline]
pub(crate) fn validate_token_interface_program(_program_id: &Address) -> Result<(), ProgramError> {
    Ok(())
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

/// `spl_token::instruction::transfer` — accounts list:
///   0. `[writable]` from
///   1. `[writable]` to
///   2. `[signer]` authority (owner/delegate)
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

/// `spl_token::instruction::transfer_checked` — adds the mint and verifies the
/// declared decimals match on-chain.
///   0. `[writable]` from
///   1. `[]` mint
///   2. `[writable]` to
///   3. `[signer]` authority
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

pub fn initialize_account<'a>(
    ctx: CpiContext<'a, InitializeAccount<'a>>,
) -> Result<(), ProgramError> {
    let ix = spl_token_2022::instruction::initialize_account(
        ctx.program,
        ctx.accounts.account.address(),
        ctx.accounts.mint.address(),
        ctx.accounts.authority.address(),
    )?;
    ctx.invoke_ix(ix)
}

pub fn initialize_account3<'a>(
    ctx: CpiContext<'a, InitializeAccount3<'a>>,
) -> Result<(), ProgramError> {
    let ix = spl_token_2022::instruction::initialize_account3(
        ctx.program,
        ctx.accounts.account.address(),
        ctx.accounts.mint.address(),
        ctx.accounts.authority.address(),
    )?;
    ctx.invoke_ix(ix)
}

pub fn initialize_mint<'a>(
    ctx: CpiContext<'a, InitializeMint<'a>>,
    decimals: u8,
    authority: &Address,
    freeze_authority: Option<&Address>,
) -> Result<(), ProgramError> {
    let ix = spl_token_2022::instruction::initialize_mint(
        ctx.program,
        ctx.accounts.mint.address(),
        authority,
        freeze_authority,
        decimals,
    )?;
    ctx.invoke_ix(ix)
}

pub fn initialize_mint2<'a>(
    ctx: CpiContext<'a, InitializeMint2<'a>>,
    decimals: u8,
    authority: &Address,
    freeze_authority: Option<&Address>,
) -> Result<(), ProgramError> {
    let ix = spl_token_2022::instruction::initialize_mint2(
        ctx.program,
        ctx.accounts.mint.address(),
        authority,
        freeze_authority,
        decimals,
    )?;
    ctx.invoke_ix(ix)
}

pub fn transfer<'a>(ctx: CpiContext<'a, Transfer<'a>>, amount: u64) -> Result<(), ProgramError> {
    #[allow(deprecated)]
    let ix = spl_token_2022::instruction::transfer(
        ctx.program,
        ctx.accounts.from.address(),
        ctx.accounts.to.address(),
        ctx.accounts.authority.address(),
        &[],
        amount,
    )?;
    ctx.invoke_ix(ix)
}

pub fn transfer_checked<'a>(
    ctx: CpiContext<'a, TransferChecked<'a>>,
    amount: u64,
    decimals: u8,
) -> Result<(), ProgramError> {
    let ix = spl_token_2022::instruction::transfer_checked(
        ctx.program,
        ctx.accounts.from.address(),
        ctx.accounts.mint.address(),
        ctx.accounts.to.address(),
        ctx.accounts.authority.address(),
        &[],
        amount,
        decimals,
    )?;
    ctx.invoke_ix(ix)
}

pub fn mint_to<'a>(ctx: CpiContext<'a, MintTo<'a>>, amount: u64) -> Result<(), ProgramError> {
    let ix = spl_token_2022::instruction::mint_to(
        ctx.program,
        ctx.accounts.mint.address(),
        ctx.accounts.to.address(),
        ctx.accounts.authority.address(),
        &[],
        amount,
    )?;
    ctx.invoke_ix(ix)
}

pub fn mint_to_checked<'a>(
    ctx: CpiContext<'a, MintToChecked<'a>>,
    amount: u64,
    decimals: u8,
) -> Result<(), ProgramError> {
    let ix = spl_token_2022::instruction::mint_to_checked(
        ctx.program,
        ctx.accounts.mint.address(),
        ctx.accounts.to.address(),
        ctx.accounts.authority.address(),
        &[],
        amount,
        decimals,
    )?;
    ctx.invoke_ix(ix)
}

pub fn burn<'a>(ctx: CpiContext<'a, Burn<'a>>, amount: u64) -> Result<(), ProgramError> {
    let ix = spl_token_2022::instruction::burn(
        ctx.program,
        ctx.accounts.from.address(),
        ctx.accounts.mint.address(),
        ctx.accounts.authority.address(),
        &[],
        amount,
    )?;
    ctx.invoke_ix(ix)
}

pub fn burn_checked<'a>(
    ctx: CpiContext<'a, BurnChecked<'a>>,
    amount: u64,
    decimals: u8,
) -> Result<(), ProgramError> {
    let ix = spl_token_2022::instruction::burn_checked(
        ctx.program,
        ctx.accounts.from.address(),
        ctx.accounts.mint.address(),
        ctx.accounts.authority.address(),
        &[],
        amount,
        decimals,
    )?;
    ctx.invoke_ix(ix)
}

pub fn approve<'a>(ctx: CpiContext<'a, Approve<'a>>, amount: u64) -> Result<(), ProgramError> {
    let ix = spl_token_2022::instruction::approve(
        ctx.program,
        ctx.accounts.to.address(),
        ctx.accounts.delegate.address(),
        ctx.accounts.authority.address(),
        &[],
        amount,
    )?;
    ctx.invoke_ix(ix)
}

pub fn approve_checked<'a>(
    ctx: CpiContext<'a, ApproveChecked<'a>>,
    amount: u64,
    decimals: u8,
) -> Result<(), ProgramError> {
    let ix = spl_token_2022::instruction::approve_checked(
        ctx.program,
        ctx.accounts.to.address(),
        ctx.accounts.mint.address(),
        ctx.accounts.delegate.address(),
        ctx.accounts.authority.address(),
        &[],
        amount,
        decimals,
    )?;
    ctx.invoke_ix(ix)
}

pub fn revoke<'a>(ctx: CpiContext<'a, Revoke<'a>>) -> Result<(), ProgramError> {
    let ix = spl_token_2022::instruction::revoke(
        ctx.program,
        ctx.accounts.source.address(),
        ctx.accounts.authority.address(),
        &[],
    )?;
    ctx.invoke_ix(ix)
}

pub fn set_authority<'a>(
    ctx: CpiContext<'a, SetAuthority<'a>>,
    authority_type: spl_token_2022::instruction::AuthorityType,
    new_authority: Option<&Address>,
) -> Result<(), ProgramError> {
    let ix = spl_token_2022::instruction::set_authority(
        ctx.program,
        ctx.accounts.account_or_mint.address(),
        new_authority,
        authority_type,
        ctx.accounts.current_authority.address(),
        &[],
    )?;
    ctx.invoke_ix(ix)
}

pub fn close_account<'a>(ctx: CpiContext<'a, CloseAccount<'a>>) -> Result<(), ProgramError> {
    let ix = spl_token_2022::instruction::close_account(
        ctx.program,
        ctx.accounts.account.address(),
        ctx.accounts.destination.address(),
        ctx.accounts.authority.address(),
        &[],
    )?;
    ctx.invoke_ix(ix)
}

pub fn freeze_account<'a>(ctx: CpiContext<'a, FreezeAccount<'a>>) -> Result<(), ProgramError> {
    let ix = spl_token_2022::instruction::freeze_account(
        ctx.program,
        ctx.accounts.account.address(),
        ctx.accounts.mint.address(),
        ctx.accounts.authority.address(),
        &[],
    )?;
    ctx.invoke_ix(ix)
}

pub fn thaw_account<'a>(ctx: CpiContext<'a, ThawAccount<'a>>) -> Result<(), ProgramError> {
    let ix = spl_token_2022::instruction::thaw_account(
        ctx.program,
        ctx.accounts.account.address(),
        ctx.accounts.mint.address(),
        ctx.accounts.authority.address(),
        &[],
    )?;
    ctx.invoke_ix(ix)
}

pub fn sync_native<'a>(ctx: CpiContext<'a, SyncNative<'a>>) -> Result<(), ProgramError> {
    let ix = spl_token_2022::instruction::sync_native(ctx.program, ctx.accounts.account.address())?;
    ctx.invoke_ix(ix)
}
