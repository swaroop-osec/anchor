use {
    super::common::validate_token_2022_program,
    crate::token_2022::spl_token_2022,
    alloc::{vec, vec::Vec},
    anchor_lang_v2::{CpiContext, CpiHandle, CpiHandleMut, ToCpiAccounts},
    pinocchio::{address::Address, instruction::InstructionAccount},
    solana_program_error::ProgramError,
};

pub struct PausableInitialize<'a> {
    pub mint: CpiHandleMut<'a>,
}

impl<'a> ToCpiAccounts<'a> for PausableInitialize<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.mint.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint.into()]
    }
}

pub struct PausableToggle<'a> {
    pub mint: CpiHandleMut<'a>,
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
        vec![self.mint.into(), self.authority]
    }
}

pub fn pausable_initialize<'a>(
    ctx: CpiContext<'a, PausableInitialize<'a>>,
    authority: &Address,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let ix = spl_token_2022::extension::pausable::instruction::initialize(
        ctx.program,
        ctx.accounts.mint.address(),
        authority,
    )?;
    ctx.invoke_ix(ix)
}

pub fn pausable_pause<'a>(ctx: CpiContext<'a, PausableToggle<'a>>) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let ix = spl_token_2022::extension::pausable::instruction::pause(
        ctx.program,
        ctx.accounts.mint.address(),
        ctx.accounts.authority.address(),
        &[],
    )?;
    ctx.invoke_ix(ix)
}

pub fn pausable_resume<'a>(ctx: CpiContext<'a, PausableToggle<'a>>) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let ix = spl_token_2022::extension::pausable::instruction::resume(
        ctx.program,
        ctx.accounts.mint.address(),
        ctx.accounts.authority.address(),
        &[],
    )?;
    ctx.invoke_ix(ix)
}
