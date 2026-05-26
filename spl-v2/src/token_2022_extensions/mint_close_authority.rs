use {
    super::common::validate_token_2022_program,
    crate::token_2022::spl_token_2022,
    alloc::{vec, vec::Vec},
    anchor_lang_v2::{CpiContext, CpiHandle, CpiHandleMut, ToCpiAccounts},
    pinocchio::{address::Address, instruction::InstructionAccount},
    solana_program_error::ProgramError,
};

pub struct MintCloseAuthorityInitialize<'a> {
    pub mint: CpiHandleMut<'a>,
}

impl<'a> ToCpiAccounts<'a> for MintCloseAuthorityInitialize<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.mint.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint.into()]
    }
}

pub fn mint_close_authority_initialize<'a>(
    ctx: CpiContext<'a, MintCloseAuthorityInitialize<'a>>,
    close_authority: Option<&Address>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let program = *ctx.program;
    let close_authority = close_authority.copied();
    let ix = spl_token_2022::instruction::initialize_mint_close_authority(
        &program,
        ctx.accounts.mint.address(),
        close_authority.as_ref(),
    )?;
    ctx.invoke_ix(ix)
}
