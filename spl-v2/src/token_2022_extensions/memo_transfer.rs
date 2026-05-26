use {
    super::common::validate_token_2022_program,
    crate::token_2022::spl_token_2022,
    alloc::{vec, vec::Vec},
    anchor_lang_v2::{CpiContext, CpiHandle, ToCpiAccounts},
    pinocchio::instruction::InstructionAccount,
    solana_program_error::ProgramError,
};

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

pub fn memo_transfer_initialize<'a>(
    ctx: CpiContext<'a, MemoTransfer<'a>>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let program = *ctx.program;
    let ix = spl_token_2022::extension::memo_transfer::instruction::enable_required_transfer_memos(
        &program,
        ctx.accounts.account.address(),
        ctx.accounts.owner.address(),
        &[],
    )?;
    ctx.invoke_ix(ix)
}

pub fn memo_transfer_disable<'a>(
    ctx: CpiContext<'a, MemoTransfer<'a>>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let program = *ctx.program;
    let ix =
        spl_token_2022::extension::memo_transfer::instruction::disable_required_transfer_memos(
            &program,
            ctx.accounts.account.address(),
            ctx.accounts.owner.address(),
            &[],
        )?;
    ctx.invoke_ix(ix)
}
