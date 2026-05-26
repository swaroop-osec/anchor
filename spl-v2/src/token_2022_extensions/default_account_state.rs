use {
    super::common::validate_token_2022_program,
    crate::token_2022::spl_token_2022,
    alloc::{vec, vec::Vec},
    anchor_lang_v2::{CpiContext, CpiHandle, CpiHandleMut, ToCpiAccounts},
    pinocchio::instruction::InstructionAccount,
    solana_program_error::ProgramError,
};

pub struct DefaultAccountStateInitialize<'a> {
    pub mint: CpiHandleMut<'a>,
}

impl<'a> ToCpiAccounts<'a> for DefaultAccountStateInitialize<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.mint.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint.into()]
    }
}

pub struct DefaultAccountStateUpdate<'a> {
    pub mint: CpiHandleMut<'a>,
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
        vec![self.mint.into(), self.freeze_authority]
    }
}

pub fn default_account_state_initialize<'a>(
    ctx: CpiContext<'a, DefaultAccountStateInitialize<'a>>,
    state: &spl_token_2022::state::AccountState,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let program = *ctx.program;
    let ix =
        spl_token_2022::extension::default_account_state::instruction::initialize_default_account_state(
            &program,
            ctx.accounts.mint.address(),
            state,
        )?;
    ctx.invoke_ix(ix)
}

pub fn default_account_state_update<'a>(
    ctx: CpiContext<'a, DefaultAccountStateUpdate<'a>>,
    state: &spl_token_2022::state::AccountState,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let program = *ctx.program;
    let ix =
        spl_token_2022::extension::default_account_state::instruction::update_default_account_state(
            &program,
            ctx.accounts.mint.address(),
            ctx.accounts.freeze_authority.address(),
            &[],
            state,
        )?;
    ctx.invoke_ix(ix)
}
