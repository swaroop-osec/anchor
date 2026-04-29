//! Repro: const-item seed expressions emit empty IDL entries.
//!
//! Buggy site: `lang-v2/derive/src/idl.rs:560-677`. When `classify_seed_value`
//! cannot resolve a seed expression to a literal at macro time, it
//! `eprintln!`s a warning and returns `const_seed_value(&[])` (empty
//! bytes). Runtime PDA derivation still works — only the IDL is degraded.

use anchor_lang_v2::prelude::*;

declare_id!("BugConstSeed11111111111111111111111111111111");

pub const MY_SEED: &[u8] = b"hello";

#[program]
pub mod idl_const_seed {
    use super::*;

    pub fn touch(_ctx: &mut Context<TouchAccounts>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct TouchAccounts {
    #[account(seeds = [MY_SEED], bump)]
    pub data: UncheckedAccount,
}
