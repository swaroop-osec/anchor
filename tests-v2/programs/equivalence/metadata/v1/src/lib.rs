use {
    anchor_lang::prelude::*,
    anchor_spl::metadata::{self, Metadata},
};

declare_id!("8HTToHNV33ciwFpvw5t1BKWhnHkQP58LVXLzEtdSS8qi");

#[program]
pub mod equivalence_metadata_v1 {
    use super::*;

    pub fn sign_metadata(ctx: Context<SignMetadataProxy>) -> Result<()> {
        let cpi_accounts = metadata::SignMetadata {
            metadata: ctx.accounts.metadata.to_account_info(),
            creator: ctx.accounts.creator.to_account_info(),
        };
        metadata::sign_metadata(CpiContext::new(
            ctx.accounts.metadata_program.key(),
            cpi_accounts,
        ))
    }

    pub fn remove_creator_verification(ctx: Context<RemoveCreatorVerificationProxy>) -> Result<()> {
        let cpi_accounts = metadata::RemoveCreatorVerification {
            metadata: ctx.accounts.metadata.to_account_info(),
            creator: ctx.accounts.creator.to_account_info(),
        };
        metadata::remove_creator_verification(CpiContext::new(
            ctx.accounts.metadata_program.key(),
            cpi_accounts,
        ))
    }

    pub fn update_primary_sale_happened_via_token(
        ctx: Context<UpdatePrimarySaleHappenedViaTokenProxy>,
    ) -> Result<()> {
        let cpi_accounts = metadata::UpdatePrimarySaleHappenedViaToken {
            metadata: ctx.accounts.metadata.to_account_info(),
            owner: ctx.accounts.owner.to_account_info(),
            token: ctx.accounts.token.to_account_info(),
        };
        metadata::update_primary_sale_happened_via_token(CpiContext::new(
            ctx.accounts.metadata_program.key(),
            cpi_accounts,
        ))
    }

    pub fn set_token_standard(ctx: Context<SetTokenStandardProxy>) -> Result<()> {
        let cpi_accounts = metadata::SetTokenStandard {
            metadata_account: ctx.accounts.metadata.to_account_info(),
            update_authority: ctx.accounts.update_authority.to_account_info(),
            mint_account: ctx.accounts.mint.to_account_info(),
        };
        metadata::set_token_standard(
            CpiContext::new(ctx.accounts.metadata_program.key(), cpi_accounts),
            None,
        )
    }
}

#[derive(Accounts)]
pub struct SignMetadataProxy<'info> {
    pub metadata_program: Program<'info, Metadata>,
    /// CHECK: forwarded to the metadata CPI helper
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: forwarded to the metadata CPI helper
    pub creator: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct RemoveCreatorVerificationProxy<'info> {
    pub metadata_program: Program<'info, Metadata>,
    /// CHECK: forwarded to the metadata CPI helper
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: forwarded to the metadata CPI helper
    pub creator: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct UpdatePrimarySaleHappenedViaTokenProxy<'info> {
    pub metadata_program: Program<'info, Metadata>,
    /// CHECK: forwarded to the metadata CPI helper
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: forwarded to the metadata CPI helper
    pub owner: UncheckedAccount<'info>,
    /// CHECK: forwarded to the metadata CPI helper
    pub token: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct SetTokenStandardProxy<'info> {
    pub metadata_program: Program<'info, Metadata>,
    /// CHECK: forwarded to the metadata CPI helper
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: forwarded to the metadata CPI helper
    pub update_authority: UncheckedAccount<'info>,
    /// CHECK: forwarded to the metadata CPI helper
    pub mint: UncheckedAccount<'info>,
}
