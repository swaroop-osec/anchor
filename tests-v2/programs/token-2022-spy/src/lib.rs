use anchor_lang_v2::prelude::*;

declare_id!("7TdHZyhueZP4B8fvbgvbGPTH4bijkBPtpWc3wBfTmWQv");

#[program]
pub mod token_2022_spy {
    use super::*;

    #[discrim = 34]
    pub fn cpi_guard(ctx: &mut Context<CpiGuard>, op: u8) -> Result<()> {
        require!(op <= 1, ProgramError::InvalidInstructionData);
        require!(
            ctx.accounts.account.account().is_writable(),
            ProgramError::InvalidAccountData
        );
        require!(
            ctx.accounts.owner.account().is_signer(),
            ProgramError::MissingRequiredSignature
        );
        Ok(())
    }

    #[discrim = 40]
    pub fn group_pointer_update(
        ctx: &mut Context<GroupPointerUpdate>,
        op: u8,
        _group_address: Address,
    ) -> Result<()> {
        require_eq!(op, 1u8, ProgramError::InvalidInstructionData);
        require!(
            ctx.accounts.mint.account().is_writable(),
            ProgramError::InvalidAccountData
        );
        require!(
            ctx.accounts.authority.account().is_signer(),
            ProgramError::MissingRequiredSignature
        );
        Ok(())
    }

    #[discrim = 41]
    pub fn group_member_pointer_update(
        ctx: &mut Context<GroupMemberPointerUpdate>,
        op: u8,
        _member_address: Address,
    ) -> Result<()> {
        require_eq!(op, 1u8, ProgramError::InvalidInstructionData);
        require!(
            ctx.accounts.mint.account().is_writable(),
            ProgramError::InvalidAccountData
        );
        require!(
            ctx.accounts.authority.account().is_signer(),
            ProgramError::MissingRequiredSignature
        );
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CpiGuard {
    #[account(mut)]
    pub account: UncheckedAccount,
    pub owner: Signer,
}

#[derive(Accounts)]
pub struct GroupPointerUpdate {
    #[account(mut)]
    pub mint: UncheckedAccount,
    pub authority: Signer,
}

#[derive(Accounts)]
pub struct GroupMemberPointerUpdate {
    #[account(mut)]
    pub mint: UncheckedAccount,
    pub authority: Signer,
}
