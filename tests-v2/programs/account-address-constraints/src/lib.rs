//! Compile-time coverage for account-like field refs in SPL constraints.
//!
//! The important cases are boxed and optional account wrappers used as
//! `mint::authority` / `token::authority` sibling fields. These compile only
//! when derive-generated constraint code projects the account address through
//! `anchor_lang_v2::AccountAddress` rather than raw `AsRef`.

use {
    anchor_lang_v2::prelude::*,
    anchor_spl_v2::{
        mint::{self, Mint},
        token::{self, Token, TokenAccount},
    },
};

declare_id!("Con9ukTn9BRPXWcjS2UBbuN3NnCwy1hcaDNZ9Hb8QMNp");

#[account]
pub struct AuthorityData {
    pub value: u64,
}

#[program]
pub mod account_address_constraints {
    use super::*;

    #[discrim = 0]
    pub fn init_mint_with_box_authority(
        _ctx: &mut Context<InitMintWithBoxAuthority>,
    ) -> Result<()> {
        Ok(())
    }

    #[discrim = 1]
    pub fn init_token_with_box_authority(
        _ctx: &mut Context<InitTokenWithBoxAuthority>,
    ) -> Result<()> {
        Ok(())
    }

    #[discrim = 2]
    pub fn check_mint_with_box_authority(
        _ctx: &mut Context<CheckMintWithBoxAuthority>,
    ) -> Result<()> {
        Ok(())
    }

    #[discrim = 3]
    pub fn check_token_with_box_authority(
        _ctx: &mut Context<CheckTokenWithBoxAuthority>,
    ) -> Result<()> {
        Ok(())
    }

    #[discrim = 4]
    pub fn check_mint_with_optional_authority(
        _ctx: &mut Context<CheckMintWithOptionalAuthority>,
    ) -> Result<()> {
        Ok(())
    }

    #[discrim = 5]
    pub fn check_token_with_optional_authority(
        _ctx: &mut Context<CheckTokenWithOptionalAuthority>,
    ) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitMintWithBoxAuthority {
    #[account(mut)]
    pub payer: Signer,
    pub authority: Box<Account<AuthorityData>>,
    #[account(
        init,
        payer = payer,
        mint::decimals = 6,
        mint::authority = authority,
    )]
    pub mint: Account<Mint>,
    pub token_program: Program<Token>,
    pub system_program: Program<System>,
}

#[derive(Accounts)]
pub struct InitTokenWithBoxAuthority {
    #[account(mut)]
    pub payer: Signer,
    pub mint: Account<Mint>,
    pub authority: Box<Account<AuthorityData>>,
    #[account(
        init,
        payer = payer,
        token::mint = mint,
        token::authority = authority,
    )]
    pub token_account: Account<TokenAccount>,
    pub token_program: Program<Token>,
    pub system_program: Program<System>,
}

#[derive(Accounts)]
pub struct CheckMintWithBoxAuthority {
    pub authority: Box<Account<AuthorityData>>,
    #[account(mut, mint::authority = authority)]
    pub mint: Account<Mint>,
}

#[derive(Accounts)]
pub struct CheckTokenWithBoxAuthority {
    pub authority: Box<Account<AuthorityData>>,
    #[account(mut, token::authority = authority)]
    pub token_account: Account<TokenAccount>,
}

#[derive(Accounts)]
pub struct CheckMintWithOptionalAuthority {
    pub authority: Option<Account<AuthorityData>>,
    #[account(mut, mint::authority = authority)]
    pub mint: Account<Mint>,
}

#[derive(Accounts)]
pub struct CheckTokenWithOptionalAuthority {
    pub authority: Option<Account<AuthorityData>>,
    #[account(mut, token::authority = authority)]
    pub token_account: Account<TokenAccount>,
}
