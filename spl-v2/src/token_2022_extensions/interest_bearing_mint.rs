use {
    super::common::validate_token_2022_program,
    crate::token_2022::spl_token_2022,
    alloc::{vec, vec::Vec},
    anchor_lang_v2::{CpiContext, CpiHandle, ToCpiAccounts},
    pinocchio::{address::Address, instruction::InstructionAccount},
    solana_program_error::ProgramError,
};

pub struct InterestBearingMintInitialize<'a> {
    pub mint: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for InterestBearingMintInitialize<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![InstructionAccount::writable(self.mint.address())]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint]
    }
}

pub struct InterestBearingMintUpdateRate<'a> {
    pub mint: CpiHandle<'a>,
    pub rate_authority: CpiHandle<'a>,
}

impl<'a> ToCpiAccounts<'a> for InterestBearingMintUpdateRate<'a> {
    fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
        vec![
            InstructionAccount::writable(self.mint.address()),
            InstructionAccount::readonly_signer(self.rate_authority.address()),
        ]
    }

    fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
        vec![self.mint, self.rate_authority]
    }
}

pub fn interest_bearing_mint_initialize<'a>(
    ctx: CpiContext<'a, InterestBearingMintInitialize<'a>>,
    rate_authority: Option<&Address>,
    rate: i16,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let program = *ctx.program;
    let ix = spl_token_2022::extension::interest_bearing_mint::instruction::initialize(
        &program,
        ctx.accounts.mint.address(),
        rate_authority.copied(),
        rate,
    )?;
    ctx.invoke_ix(ix)
}

pub fn interest_bearing_mint_update_rate<'a>(
    ctx: CpiContext<'a, InterestBearingMintUpdateRate<'a>>,
    rate: i16,
) -> Result<(), ProgramError> {
    validate_token_2022_program(ctx.program)?;
    let program = *ctx.program;
    let ix = spl_token_2022::extension::interest_bearing_mint::instruction::update_rate(
        &program,
        ctx.accounts.mint.address(),
        ctx.accounts.rate_authority.address(),
        &[],
        rate,
    )?;
    ctx.invoke_ix(ix)
}
