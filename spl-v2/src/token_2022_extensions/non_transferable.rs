use {
    super::common::validate_token_2022_program,
    crate::token_2022::spl_token_2022,
    alloc::{vec, vec::Vec},
    anchor_lang_v2::{CpiContext, CpiHandle, CpiHandleMut, ToCpiAccounts},
    pinocchio::instruction::InstructionAccount,
    solana_program_error::ProgramError,
};

pub struct NonTransferableMintInitialize<'a> {
    pub mint: CpiHandleMut<'a>,
}

impl<'a> ToCpiAccounts<'a> for NonTransferableMintInitialize<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.mint.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint.into()]
    }
}

pub fn non_transferable_mint_initialize<'a>(
    ctx: CpiContext<'a, NonTransferableMintInitialize<'a>>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let program = *ctx.program;
    let ix = spl_token_2022::instruction::initialize_non_transferable_mint(
        &program,
        ctx.accounts.mint.address(),
    )?;
    ctx.invoke_ix(ix)
}
