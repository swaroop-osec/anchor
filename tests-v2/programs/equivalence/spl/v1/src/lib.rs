use anchor_lang::__private::bytemuck::{bytes_of, Pod};
use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Mint, TokenAccount},
    token_2022::spl_token_2022::{
        extension::{
            cpi_guard::CpiGuard as SplCpiGuard,
            group_member_pointer::GroupMemberPointer as SplGroupMemberPointer,
            group_pointer::GroupPointer as SplGroupPointer,
            metadata_pointer::MetadataPointer as SplMetadataPointer,
            mint_close_authority::MintCloseAuthority as SplMintCloseAuthority,
            pausable::{
                PausableAccount as SplPausableAccount, PausableConfig as SplPausableConfig,
            },
            permanent_delegate::PermanentDelegate as SplPermanentDelegate,
            transfer_fee::{
                TransferFeeAmount as SplTransferFeeAmount,
                TransferFeeConfig as SplTransferFeeConfig,
            },
            transfer_hook::{
                TransferHook as SplTransferHook, TransferHookAccount as SplTransferHookAccount,
            },
            BaseStateWithExtensions, Extension as SplExtension, StateWithExtensions,
        },
        state::{Account as SplToken2022Account, Mint as SplToken2022Mint},
        ID as TOKEN_2022_ID,
    },
    token_interface,
};

declare_id!("HYLtHw8VKojJTZXzodkeaerYjK2um5bSrqyGwMYYTjNL");

const TAG_STRICT_MINT: u8 = 1;
const TAG_STRICT_TOKEN: u8 = 2;
const TAG_INTERFACE_MINT: u8 = 3;
const TAG_INTERFACE_TOKEN: u8 = 4;
const TAG_INTERFACE_MINT_EXTENSION: u8 = 5;
const TAG_INTERFACE_TOKEN_EXTENSION: u8 = 6;
const EXTENSION_STATUS_FOUND: u8 = 1;
const EXTENSION_STATUS_ILLEGAL_OWNER: u8 = 2;
const EXTENSION_STATUS_ACCESS_ERROR: u8 = 3;

#[program]
pub mod equivalence_spl_v1 {
    use super::*;

    pub fn check_strict_mint(ctx: Context<CheckStrictMint>) -> Result<()> {
        write_mint_observation(
            &ctx.accounts.out,
            TAG_STRICT_MINT,
            ctx.accounts.mint.supply,
            ctx.accounts.mint.decimals,
            ctx.accounts.mint.is_initialized,
            coption_key(ctx.accounts.mint.mint_authority),
            coption_key(ctx.accounts.mint.freeze_authority),
        )
    }

    pub fn check_strict_token_account(ctx: Context<CheckStrictTokenAccount>) -> Result<()> {
        write_token_account_observation(
            &ctx.accounts.out,
            TAG_STRICT_TOKEN,
            ctx.accounts.token_account.mint.to_bytes(),
            ctx.accounts.token_account.owner.to_bytes(),
            ctx.accounts.token_account.amount,
            coption_key(ctx.accounts.token_account.delegate),
            ctx.accounts.token_account.state as u8,
            coption_u64(ctx.accounts.token_account.is_native),
            ctx.accounts.token_account.delegated_amount,
            coption_key(ctx.accounts.token_account.close_authority),
        )
    }

    pub fn check_interface_mint(ctx: Context<CheckInterfaceMint>) -> Result<()> {
        write_mint_observation(
            &ctx.accounts.out,
            TAG_INTERFACE_MINT,
            ctx.accounts.mint.supply,
            ctx.accounts.mint.decimals,
            ctx.accounts.mint.is_initialized,
            coption_key(ctx.accounts.mint.mint_authority),
            coption_key(ctx.accounts.mint.freeze_authority),
        )
    }

    pub fn check_interface_token_account(ctx: Context<CheckInterfaceTokenAccount>) -> Result<()> {
        write_token_account_observation(
            &ctx.accounts.out,
            TAG_INTERFACE_TOKEN,
            ctx.accounts.token_account.mint.to_bytes(),
            ctx.accounts.token_account.owner.to_bytes(),
            ctx.accounts.token_account.amount,
            coption_key(ctx.accounts.token_account.delegate),
            ctx.accounts.token_account.state as u8,
            coption_u64(ctx.accounts.token_account.is_native),
            ctx.accounts.token_account.delegated_amount,
            coption_key(ctx.accounts.token_account.close_authority),
        )
    }

    pub fn check_interface_mint_extension(
        ctx: Context<CheckInterfaceMintExtension>,
        operation: u8,
    ) -> Result<()> {
        let mint = ctx.accounts.mint.to_account_info();
        match operation {
            0 => write_mint_extension_observation::<SplMetadataPointer>(
                &mint,
                &ctx.accounts.out,
                operation,
            ),
            1 => write_mint_extension_observation::<SplGroupPointer>(
                &mint,
                &ctx.accounts.out,
                operation,
            ),
            2 => write_mint_extension_observation::<SplGroupMemberPointer>(
                &mint,
                &ctx.accounts.out,
                operation,
            ),
            3 => write_mint_extension_observation::<SplTransferHook>(
                &mint,
                &ctx.accounts.out,
                operation,
            ),
            4 => write_mint_extension_observation::<SplMintCloseAuthority>(
                &mint,
                &ctx.accounts.out,
                operation,
            ),
            5 => write_mint_extension_observation::<SplPermanentDelegate>(
                &mint,
                &ctx.accounts.out,
                operation,
            ),
            6 => write_mint_extension_observation::<SplTransferFeeConfig>(
                &mint,
                &ctx.accounts.out,
                operation,
            ),
            7 => write_mint_extension_observation::<SplPausableConfig>(
                &mint,
                &ctx.accounts.out,
                operation,
            ),
            _ => write_extension_observation(
                &ctx.accounts.out,
                TAG_INTERFACE_MINT_EXTENSION,
                operation,
                EXTENSION_STATUS_ACCESS_ERROR,
                &[],
            ),
        }
    }

    pub fn check_interface_token_account_extension(
        ctx: Context<CheckInterfaceTokenAccountExtension>,
        operation: u8,
    ) -> Result<()> {
        let token_account = ctx.accounts.token_account.to_account_info();
        match operation {
            0 => write_token_account_extension_observation::<SplTransferFeeAmount>(
                &token_account,
                &ctx.accounts.out,
                operation,
            ),
            1 => write_token_account_extension_observation::<SplCpiGuard>(
                &token_account,
                &ctx.accounts.out,
                operation,
            ),
            2 => write_token_account_extension_observation::<SplTransferHookAccount>(
                &token_account,
                &ctx.accounts.out,
                operation,
            ),
            3 => write_token_account_extension_observation::<SplPausableAccount>(
                &token_account,
                &ctx.accounts.out,
                operation,
            ),
            _ => write_extension_observation(
                &ctx.accounts.out,
                TAG_INTERFACE_TOKEN_EXTENSION,
                operation,
                EXTENSION_STATUS_ACCESS_ERROR,
                &[],
            ),
        }
    }
}

#[derive(Accounts)]
pub struct CheckStrictMint<'info> {
    pub mint: Account<'info, Mint>,
    /// CHECK: pre-owned scratch account used only for deterministic output
    #[account(mut)]
    pub out: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct CheckStrictTokenAccount<'info> {
    pub token_account: Account<'info, TokenAccount>,
    /// CHECK: pre-owned scratch account used only for deterministic output
    #[account(mut)]
    pub out: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct CheckInterfaceMint<'info> {
    pub mint: InterfaceAccount<'info, token_interface::Mint>,
    /// CHECK: pre-owned scratch account used only for deterministic output
    #[account(mut)]
    pub out: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct CheckInterfaceTokenAccount<'info> {
    pub token_account: InterfaceAccount<'info, token_interface::TokenAccount>,
    /// CHECK: pre-owned scratch account used only for deterministic output
    #[account(mut)]
    pub out: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct CheckInterfaceMintExtension<'info> {
    pub mint: InterfaceAccount<'info, token_interface::Mint>,
    /// CHECK: pre-owned scratch account used only for deterministic output
    #[account(mut)]
    pub out: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct CheckInterfaceTokenAccountExtension<'info> {
    pub token_account: InterfaceAccount<'info, token_interface::TokenAccount>,
    /// CHECK: pre-owned scratch account used only for deterministic output
    #[account(mut)]
    pub out: UncheckedAccount<'info>,
}

fn coption_key(
    value: anchor_lang::solana_program::program_option::COption<Pubkey>,
) -> Option<[u8; 32]> {
    match value {
        anchor_lang::solana_program::program_option::COption::Some(key) => Some(key.to_bytes()),
        anchor_lang::solana_program::program_option::COption::None => None,
    }
}

fn coption_u64(value: anchor_lang::solana_program::program_option::COption<u64>) -> Option<u64> {
    match value {
        anchor_lang::solana_program::program_option::COption::Some(value) => Some(value),
        anchor_lang::solana_program::program_option::COption::None => None,
    }
}

fn write_mint_observation(
    out: &UncheckedAccount<'_>,
    tag: u8,
    supply: u64,
    decimals: u8,
    is_initialized: bool,
    mint_authority: Option<[u8; 32]>,
    freeze_authority: Option<[u8; 32]>,
) -> Result<()> {
    let mut data = out.try_borrow_mut_data()?;
    clear(&mut data);
    data[0] = tag;
    data[1..9].copy_from_slice(&supply.to_le_bytes());
    data[9] = decimals;
    data[10] = is_initialized as u8;
    write_option_key(&mut data, 11, mint_authority);
    write_option_key(&mut data, 44, freeze_authority);
    Ok(())
}

fn write_token_account_observation(
    out: &UncheckedAccount<'_>,
    tag: u8,
    mint: [u8; 32],
    owner: [u8; 32],
    amount: u64,
    delegate: Option<[u8; 32]>,
    state: u8,
    is_native: Option<u64>,
    delegated_amount: u64,
    close_authority: Option<[u8; 32]>,
) -> Result<()> {
    let mut data = out.try_borrow_mut_data()?;
    clear(&mut data);
    data[0] = tag;
    data[1..33].copy_from_slice(&mint);
    data[33..65].copy_from_slice(&owner);
    data[65..73].copy_from_slice(&amount.to_le_bytes());
    write_option_key(&mut data, 73, delegate);
    data[106] = state;
    write_option_u64(&mut data, 107, is_native);
    data[116..124].copy_from_slice(&delegated_amount.to_le_bytes());
    write_option_key(&mut data, 124, close_authority);
    Ok(())
}

fn write_mint_extension_observation<T>(
    mint: &AccountInfo<'_>,
    out: &UncheckedAccount<'_>,
    operation: u8,
) -> Result<()>
where
    T: SplExtension + Pod + Copy,
{
    if mint.owner.as_ref() != TOKEN_2022_ID.as_ref() {
        return write_extension_observation(
            out,
            TAG_INTERFACE_MINT_EXTENSION,
            operation,
            EXTENSION_STATUS_ILLEGAL_OWNER,
            &[],
        );
    }

    let data = mint.try_borrow_data()?;
    match StateWithExtensions::<SplToken2022Mint>::unpack(&data)
        .and_then(|state| state.get_extension::<T>().copied())
    {
        Ok(extension) => write_extension_observation(
            out,
            TAG_INTERFACE_MINT_EXTENSION,
            operation,
            EXTENSION_STATUS_FOUND,
            bytes_of(&extension),
        ),
        Err(_) => write_extension_observation(
            out,
            TAG_INTERFACE_MINT_EXTENSION,
            operation,
            EXTENSION_STATUS_ACCESS_ERROR,
            &[],
        ),
    }
}

fn write_token_account_extension_observation<T>(
    token_account: &AccountInfo<'_>,
    out: &UncheckedAccount<'_>,
    operation: u8,
) -> Result<()>
where
    T: SplExtension + Pod + Copy,
{
    if token_account.owner.as_ref() != TOKEN_2022_ID.as_ref() {
        return write_extension_observation(
            out,
            TAG_INTERFACE_TOKEN_EXTENSION,
            operation,
            EXTENSION_STATUS_ILLEGAL_OWNER,
            &[],
        );
    }

    let data = token_account.try_borrow_data()?;
    match StateWithExtensions::<SplToken2022Account>::unpack(&data)
        .and_then(|state| state.get_extension::<T>().copied())
    {
        Ok(extension) => write_extension_observation(
            out,
            TAG_INTERFACE_TOKEN_EXTENSION,
            operation,
            EXTENSION_STATUS_FOUND,
            bytes_of(&extension),
        ),
        Err(_) => write_extension_observation(
            out,
            TAG_INTERFACE_TOKEN_EXTENSION,
            operation,
            EXTENSION_STATUS_ACCESS_ERROR,
            &[],
        ),
    }
}

fn write_extension_observation(
    out: &UncheckedAccount<'_>,
    tag: u8,
    operation: u8,
    status: u8,
    extension_data: &[u8],
) -> Result<()> {
    let mut data = out.try_borrow_mut_data()?;
    clear(&mut data);
    data[0] = tag;
    data[1] = operation;
    data[2] = status;
    data[3..5].copy_from_slice(&(extension_data.len() as u16).to_le_bytes());
    data[5..5 + extension_data.len()].copy_from_slice(extension_data);
    Ok(())
}

fn clear(data: &mut [u8]) {
    for byte in data.iter_mut() {
        *byte = 0;
    }
}

fn write_option_key(data: &mut [u8], offset: usize, value: Option<[u8; 32]>) {
    if let Some(key) = value {
        data[offset] = 1;
        data[offset + 1..offset + 33].copy_from_slice(&key);
    }
}

fn write_option_u64(data: &mut [u8], offset: usize, value: Option<u64>) {
    if let Some(value) = value {
        data[offset] = 1;
        data[offset + 1..offset + 9].copy_from_slice(&value.to_le_bytes());
    }
}
