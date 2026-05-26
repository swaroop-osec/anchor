use {
    super::common::validate_token_2022_program,
    alloc::{vec, vec::Vec},
    anchor_lang_v2::{CpiContext, CpiHandle, CpiHandleMut, ToCpiAccounts},
    pinocchio::{address::Address, instruction::InstructionAccount},
    solana_program_error::ProgramError,
};

pub struct TokenGroupInitialize<'a> {
    pub group: CpiHandleMut<'a>,
    pub mint: CpiHandle<'a>,
    pub mint_authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for TokenGroupInitialize<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.group.address()),
            InstructionAccount::new(self.mint.address(), false, false),
            InstructionAccount::readonly_signer(self.mint_authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.group.into(), self.mint, self.mint_authority]
    }
}

pub struct TokenMemberInitialize<'a> {
    pub member: CpiHandleMut<'a>,
    pub member_mint: CpiHandle<'a>,
    pub member_mint_authority: CpiHandle<'a>,
    pub group: CpiHandleMut<'a>,
    pub group_update_authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for TokenMemberInitialize<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.member.address()),
            InstructionAccount::new(self.member_mint.address(), false, false),
            InstructionAccount::readonly_signer(self.member_mint_authority.address()),
            InstructionAccount::writable(self.group.address()),
            InstructionAccount::readonly_signer(self.group_update_authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![
            self.member.into(),
            self.member_mint,
            self.member_mint_authority,
            self.group.into(),
            self.group_update_authority,
        ]
    }
}

pub fn token_group_initialize<'a>(
    ctx: CpiContext<'a, TokenGroupInitialize<'a>>,
    update_authority: Option<&Address>,
    max_size: u64,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let program = *ctx.program;
    let ix = spl_token_group_interface::instruction::initialize_group(
        &program,
        ctx.accounts.group.address(),
        ctx.accounts.mint.address(),
        ctx.accounts.mint_authority.address(),
        update_authority.copied(),
        max_size,
    );
    ctx.invoke_ix(ix)
}

pub fn token_member_initialize<'a>(
    ctx: CpiContext<'a, TokenMemberInitialize<'a>>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let program = *ctx.program;
    let ix = spl_token_group_interface::instruction::initialize_member(
        &program,
        ctx.accounts.member.address(),
        ctx.accounts.member_mint.address(),
        ctx.accounts.member_mint_authority.address(),
        ctx.accounts.group.address(),
        ctx.accounts.group_update_authority.address(),
    );
    ctx.invoke_ix(ix)
}
