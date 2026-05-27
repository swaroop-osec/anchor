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

    #[discrim = 29]
    pub fn reallocate(ctx: &mut Context<Reallocate>, extension_type: u16) -> Result<()> {
        require_eq!(extension_type, 20u16, ProgramError::InvalidInstructionData);
        require!(
            ctx.accounts.account.account().is_writable(),
            ProgramError::InvalidAccountData
        );
        require!(
            ctx.accounts.payer.account().is_writable(),
            ProgramError::InvalidAccountData
        );
        require!(
            ctx.accounts.payer.account().is_signer(),
            ProgramError::MissingRequiredSignature
        );
        require!(
            !ctx.accounts.system_program.account().is_writable(),
            ProgramError::InvalidAccountData
        );
        require!(
            ctx.accounts.authority.account().is_signer(),
            ProgramError::MissingRequiredSignature
        );
        Ok(())
    }

    #[discrim = 21]
    pub fn get_account_data_size(
        ctx: &mut Context<ReturnDataMint>,
        first_extension: u16,
        second_extension: u16,
    ) -> Result<()> {
        require_eq!(first_extension, 7u16, ProgramError::InvalidInstructionData);
        require_eq!(second_extension, 9u16, ProgramError::InvalidInstructionData);
        require!(
            !ctx.accounts.mint.account().is_writable(),
            ProgramError::InvalidAccountData
        );
        anchor_lang_v2::solana_program::program::set_return_data(&4242u64.to_le_bytes());
        Ok(())
    }

    #[discrim = 23]
    pub fn amount_to_ui_amount(ctx: &mut Context<ReturnDataMint>, amount: u64) -> Result<()> {
        require!(
            !ctx.accounts.mint.account().is_writable(),
            ProgramError::InvalidAccountData
        );
        match amount {
            123456789 => anchor_lang_v2::solana_program::program::set_return_data(b"1234.56789"),
            987654321 => anchor_lang_v2::solana_program::program::set_return_data(&[0xff, 0xfe]),
            _ => return Err(ProgramError::InvalidInstructionData.into()),
        }
        Ok(())
    }

    #[discrim = 24]
    pub fn ui_amount_to_amount(
        ctx: &mut Context<ReturnDataMint>,
        b0: u8,
        b1: u8,
        b2: u8,
        b3: u8,
        b4: u8,
        b5: u8,
    ) -> Result<()> {
        require!(
            !ctx.accounts.mint.account().is_writable(),
            ProgramError::InvalidAccountData
        );
        match [b0, b1, b2, b3, b4, b5] {
            [b'4', b'2', b'.', b'1', b'2', b'5'] => {
                anchor_lang_v2::solana_program::program::set_return_data(&42125u64.to_le_bytes())
            }
            [b'b', b'a', b'd', b'r', b'e', b't'] => {
                anchor_lang_v2::solana_program::program::set_return_data(&[1, 2, 3])
            }
            _ => return Err(ProgramError::InvalidInstructionData.into()),
        }
        Ok(())
    }

    #[discrim = 22]
    pub fn initialize_immutable_owner(ctx: &mut Context<InitializeImmutableOwner>) -> Result<()> {
        require!(
            ctx.accounts.token_account.account().is_writable(),
            ProgramError::InvalidAccountData
        );
        Ok(())
    }

    #[discrim = 32]
    pub fn initialize_non_transferable_mint(
        ctx: &mut Context<InitializeNonTransferableMint>,
    ) -> Result<()> {
        require!(
            ctx.accounts.mint.account().is_writable(),
            ProgramError::InvalidAccountData
        );
        Ok(())
    }

    #[discrim = 25]
    pub fn initialize_mint_close_authority(
        ctx: &mut Context<InitializeMintCloseAuthority>,
        option_tag: u8,
        close_authority: Address,
    ) -> Result<()> {
        require_eq!(option_tag, 1u8, ProgramError::InvalidInstructionData);
        require!(
            close_authority.as_ref().iter().any(|byte| *byte != 0),
            ProgramError::InvalidInstructionData
        );
        require!(
            ctx.accounts.mint.account().is_writable(),
            ProgramError::InvalidAccountData
        );
        Ok(())
    }

    #[discrim = 35]
    pub fn initialize_permanent_delegate(
        ctx: &mut Context<InitializePermanentDelegate>,
        delegate: Address,
    ) -> Result<()> {
        require!(
            delegate.as_ref().iter().any(|byte| *byte != 0),
            ProgramError::InvalidInstructionData
        );
        require!(
            ctx.accounts.mint.account().is_writable(),
            ProgramError::InvalidAccountData
        );
        Ok(())
    }

    #[discrim = 28]
    pub fn default_account_state_initialize(
        ctx: &mut Context<InitializeExtensionMint>,
        op: u8,
        state: u8,
    ) -> Result<()> {
        require_eq!(op, 0u8, ProgramError::InvalidInstructionData);
        require_eq!(state, 2u8, ProgramError::InvalidInstructionData);
        require!(
            ctx.accounts.mint.account().is_writable(),
            ProgramError::InvalidAccountData
        );
        Ok(())
    }

    #[discrim = 30]
    pub fn memo_transfer_enable(ctx: &mut Context<MemoTransfer>, op: u8) -> Result<()> {
        require_eq!(op, 0u8, ProgramError::InvalidInstructionData);
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

    #[discrim = 39]
    pub fn metadata_pointer_initialize(
        ctx: &mut Context<InitializeExtensionMint>,
        op: u8,
        authority: Address,
        metadata_address: Address,
    ) -> Result<()> {
        require_eq!(op, 0u8, ProgramError::InvalidInstructionData);
        require!(
            authority.as_ref().iter().any(|byte| *byte != 0),
            ProgramError::InvalidInstructionData
        );
        require!(
            metadata_address.as_ref().iter().any(|byte| *byte != 0),
            ProgramError::InvalidInstructionData
        );
        require!(
            ctx.accounts.mint.account().is_writable(),
            ProgramError::InvalidAccountData
        );
        Ok(())
    }

    #[discrim = 36]
    pub fn transfer_hook_initialize(
        ctx: &mut Context<InitializeExtensionMint>,
        op: u8,
        authority: Address,
        hook_program: Address,
    ) -> Result<()> {
        require_eq!(op, 0u8, ProgramError::InvalidInstructionData);
        require!(
            authority.as_ref().iter().any(|byte| *byte != 0),
            ProgramError::InvalidInstructionData
        );
        require!(
            hook_program.as_ref().iter().any(|byte| *byte != 0),
            ProgramError::InvalidInstructionData
        );
        require!(
            ctx.accounts.mint.account().is_writable(),
            ProgramError::InvalidAccountData
        );
        Ok(())
    }

    #[discrim = 33]
    pub fn interest_bearing_mint_initialize(
        ctx: &mut Context<InitializeExtensionMint>,
        op: u8,
        authority: Address,
        rate: i16,
    ) -> Result<()> {
        require_eq!(op, 0u8, ProgramError::InvalidInstructionData);
        require!(
            authority.as_ref().iter().any(|byte| *byte != 0),
            ProgramError::InvalidInstructionData
        );
        require_eq!(rate, 125i16, ProgramError::InvalidInstructionData);
        require!(
            ctx.accounts.mint.account().is_writable(),
            ProgramError::InvalidAccountData
        );
        Ok(())
    }

    #[discrim = 44]
    pub fn pausable_initialize(
        ctx: &mut Context<InitializeExtensionMint>,
        op: u8,
        authority: Address,
    ) -> Result<()> {
        require_eq!(op, 0u8, ProgramError::InvalidInstructionData);
        require!(
            authority.as_ref().iter().any(|byte| *byte != 0),
            ProgramError::InvalidInstructionData
        );
        require!(
            ctx.accounts.mint.account().is_writable(),
            ProgramError::InvalidAccountData
        );
        Ok(())
    }

    #[discrim = 26]
    pub fn transfer_fee_initialize(
        ctx: &mut Context<InitializeExtensionMint>,
        op: u8,
        config_authority_tag: u8,
        config_authority: Address,
        withdraw_authority_tag: u8,
        transfer_fee_basis_points: u16,
        maximum_fee: u64,
    ) -> Result<()> {
        require_eq!(op, 0u8, ProgramError::InvalidInstructionData);
        require_eq!(
            config_authority_tag,
            1u8,
            ProgramError::InvalidInstructionData
        );
        require!(
            config_authority.as_ref().iter().any(|byte| *byte != 0),
            ProgramError::InvalidInstructionData
        );
        require_eq!(
            withdraw_authority_tag,
            0u8,
            ProgramError::InvalidInstructionData
        );
        require_eq!(
            transfer_fee_basis_points,
            111u16,
            ProgramError::InvalidInstructionData
        );
        require_eq!(maximum_fee, 42u64, ProgramError::InvalidInstructionData);
        require!(
            ctx.accounts.mint.account().is_writable(),
            ProgramError::InvalidAccountData
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

#[derive(Accounts)]
pub struct Reallocate {
    #[account(mut)]
    pub account: UncheckedAccount,
    #[account(mut)]
    pub payer: Signer,
    pub system_program: UncheckedAccount,
    pub authority: Signer,
}

#[derive(Accounts)]
pub struct ReturnDataMint {
    pub mint: UncheckedAccount,
}

#[derive(Accounts)]
pub struct InitializeImmutableOwner {
    #[account(mut)]
    pub token_account: UncheckedAccount,
}

#[derive(Accounts)]
pub struct InitializeNonTransferableMint {
    #[account(mut)]
    pub mint: UncheckedAccount,
}

#[derive(Accounts)]
pub struct InitializeMintCloseAuthority {
    #[account(mut)]
    pub mint: UncheckedAccount,
}

#[derive(Accounts)]
pub struct InitializePermanentDelegate {
    #[account(mut)]
    pub mint: UncheckedAccount,
}

#[derive(Accounts)]
pub struct InitializeExtensionMint {
    #[account(mut)]
    pub mint: UncheckedAccount,
}

#[derive(Accounts)]
pub struct MemoTransfer {
    #[account(mut)]
    pub account: UncheckedAccount,
    pub owner: Signer,
}
