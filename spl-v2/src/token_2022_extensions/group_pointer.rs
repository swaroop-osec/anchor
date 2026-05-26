use {
    super::common::validate_token_2022_program,
    crate::token_2022::spl_token_2022,
    anchor_lang_v2::{CpiContext, CpiHandle, CpiHandleMut, ToCpiAccounts},
    pinocchio::address::Address,
    solana_program_error::ProgramError,
};

#[derive(ToCpiAccounts)]
pub struct GroupPointerInitialize<'a> {
    pub mint: CpiHandleMut<'a>,
}

#[derive(ToCpiAccounts)]
pub struct GroupPointerUpdate<'a> {
    pub mint: CpiHandleMut<'a>,
    #[account_meta(duplicate_readonly)]
    #[signer]
    pub authority: CpiHandle<'a>,
}

pub fn group_pointer_initialize<'a>(
    ctx: CpiContext<'a, GroupPointerInitialize<'a>>,
    authority: Option<&Address>,
    group_address: Option<&Address>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let ix = spl_token_2022::extension::group_pointer::instruction::initialize(
        ctx.program,
        ctx.accounts.mint.address(),
        authority.copied(),
        group_address.copied(),
    )?;
    ctx.invoke_ix(ix)
}

pub fn group_pointer_update<'a>(
    ctx: CpiContext<'a, GroupPointerUpdate<'a>>,
    group_address: Option<&Address>,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let ix = spl_token_2022::extension::group_pointer::instruction::update(
        ctx.program,
        ctx.accounts.mint.address(),
        ctx.accounts.authority.address(),
        &[ctx.accounts.authority.address()],
        group_address.copied(),
    )?;
    ctx.invoke_ix(ix)
}
