use {
    super::common::validate_token_2022_program,
    crate::token_2022::spl_token_2022,
    alloc::{vec, vec::Vec},
    anchor_lang_v2::{CpiContext, CpiHandle, ToCpiAccounts},
    pinocchio::instruction::InstructionAccount,
    solana_program_error::ProgramError,
};

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

pub fn immutable_owner_initialize<'a>(
    ctx: CpiContext<'a, ImmutableOwnerInitialize<'a>>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let program = *ctx.program;
    let ix = spl_token_2022::instruction::initialize_immutable_owner(
        &program,
        ctx.accounts.token_account.address(),
    )?;
    ctx.invoke_ix(ix)
}
