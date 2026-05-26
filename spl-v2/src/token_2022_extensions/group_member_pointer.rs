use {
    super::common::validate_token_2022_program,
    crate::token_2022::spl_token_2022,
    alloc::{vec, vec::Vec},
    anchor_lang_v2::{CpiContext, CpiHandle, CpiHandleMut, ToCpiAccounts},
    pinocchio::{address::Address, instruction::InstructionAccount},
    solana_program_error::ProgramError,
};

pub struct GroupMemberPointerInitialize<'a> {
    pub mint: CpiHandleMut<'a>,
}

impl<'a> ToCpiAccounts<'a> for GroupMemberPointerInitialize<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.mint.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint.into()]
    }
}

pub struct GroupMemberPointerUpdate<'a> {
    pub mint: CpiHandleMut<'a>,
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
        vec![self.mint.into(), self.authority]
    }
}

pub fn group_member_pointer_initialize<'a>(
    ctx: CpiContext<'a, GroupMemberPointerInitialize<'a>>,
    authority: Option<&Address>,
    member_address: Option<&Address>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let ix = spl_token_2022::extension::group_member_pointer::instruction::initialize(
        ctx.program,
        ctx.accounts.mint.address(),
        authority.copied(),
        member_address.copied(),
    )?;
    ctx.invoke_ix(ix)
}

pub fn group_member_pointer_update<'a>(
    ctx: CpiContext<'a, GroupMemberPointerUpdate<'a>>,
    member_address: Option<&Address>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let ix = spl_token_2022::extension::group_member_pointer::instruction::update(
        ctx.program,
        ctx.accounts.mint.address(),
        ctx.accounts.authority.address(),
        &[],
        member_address.copied(),
    )?;
    ctx.invoke_ix(ix)
}
