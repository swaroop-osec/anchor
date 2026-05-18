use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Mint, TokenAccount},
    token_interface,
};

declare_id!("HYLtHw8VKojJTZXzodkeaerYjK2um5bSrqyGwMYYTjNL");

const TAG_STRICT_MINT: u8 = 1;
const TAG_STRICT_TOKEN: u8 = 2;
const TAG_INTERFACE_MINT: u8 = 3;
const TAG_INTERFACE_TOKEN: u8 = 4;

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
