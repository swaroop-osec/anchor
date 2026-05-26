use {
    alloc::vec::Vec, pinocchio::address::Address, solana_program_error::ProgramError,
    solana_pubkey::Pubkey,
};

#[cfg(any(feature = "guardrails", test))]
use anchor_lang_v2::{programs::Token2022, Id};

#[cfg(feature = "guardrails")]
#[inline]
pub(crate) fn validate_token_2022_program(program: &Address) -> Result<(), ProgramError> {
    if *program != Token2022::id() {
        return Err(ProgramError::IncorrectProgramId);
    }
    Ok(())
}

#[cfg(not(feature = "guardrails"))]
#[inline]
pub(crate) fn validate_token_2022_program(_program: &Address) -> Result<(), ProgramError> {
    Ok(())
}

pub(crate) fn pubkey_refs(pubkeys: &[Pubkey]) -> Vec<&Pubkey> {
    pubkeys.iter().collect()
}
