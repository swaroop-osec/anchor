use {
    anchor_lang_v2::prelude::*,
    anchor_spl_v2::{token_interface, Mint, TokenAccount},
};

declare_id!("5FGXfwXAgDDy76hUXWQEYdBF8ztPezhq7AwibdDtFWvs");

const TAG_STRICT_MINT: u8 = 1;
const TAG_STRICT_TOKEN: u8 = 2;
const TAG_INTERFACE_MINT: u8 = 3;
const TAG_INTERFACE_TOKEN: u8 = 4;

#[program]
pub mod equivalence_spl_v2 {
    use super::*;

    #[discrim = 0]
    pub fn check_strict_mint(ctx: &mut Context<CheckStrictMint>) -> Result<()> {
        let mint = &*ctx.accounts.mint;
        write_mint_observation(
            &ctx.accounts.out,
            TAG_STRICT_MINT,
            mint.supply(),
            mint.decimals(),
            mint.is_initialized(),
            mint.mint_authority().map(|key| key.to_bytes()),
            mint.freeze_authority().map(|key| key.to_bytes()),
        )
    }

    #[discrim = 1]
    pub fn check_strict_token_account(ctx: &mut Context<CheckStrictTokenAccount>) -> Result<()> {
        let token_account = &*ctx.accounts.token_account;
        write_token_account_observation(
            &ctx.accounts.out,
            TAG_STRICT_TOKEN,
            token_account.mint().to_bytes(),
            token_account.owner().to_bytes(),
            token_account.amount(),
            token_account.delegate().map(|key| key.to_bytes()),
            token_account.state(),
            token_account.native_amount(),
            token_account.delegated_amount(),
            token_account.close_authority().map(|key| key.to_bytes()),
        )
    }

    #[discrim = 2]
    pub fn check_interface_mint(ctx: &mut Context<CheckInterfaceMint>) -> Result<()> {
        let mint = &*ctx.accounts.mint;
        write_mint_observation(
            &ctx.accounts.out,
            TAG_INTERFACE_MINT,
            mint.supply(),
            mint.decimals(),
            mint.is_initialized(),
            mint.mint_authority().map(|key| key.to_bytes()),
            mint.freeze_authority().map(|key| key.to_bytes()),
        )
    }

    #[discrim = 3]
    pub fn check_interface_token_account(
        ctx: &mut Context<CheckInterfaceTokenAccount>,
    ) -> Result<()> {
        let token_account = &*ctx.accounts.token_account;
        write_token_account_observation(
            &ctx.accounts.out,
            TAG_INTERFACE_TOKEN,
            token_account.mint().to_bytes(),
            token_account.owner().to_bytes(),
            token_account.amount(),
            token_account.delegate().map(|key| key.to_bytes()),
            token_account.state(),
            token_account.native_amount(),
            token_account.delegated_amount(),
            token_account.close_authority().map(|key| key.to_bytes()),
        )
    }
}

#[derive(Accounts)]
pub struct CheckStrictMint {
    pub mint: Account<Mint>,
    #[account(mut)]
    pub out: UncheckedAccount,
}

#[derive(Accounts)]
pub struct CheckStrictTokenAccount {
    pub token_account: Account<TokenAccount>,
    #[account(mut)]
    pub out: UncheckedAccount,
}

#[derive(Accounts)]
pub struct CheckInterfaceMint {
    pub mint: InterfaceAccount<token_interface::Mint>,
    #[account(mut)]
    pub out: UncheckedAccount,
}

#[derive(Accounts)]
pub struct CheckInterfaceTokenAccount {
    pub token_account: InterfaceAccount<token_interface::TokenAccount>,
    #[account(mut)]
    pub out: UncheckedAccount,
}

fn write_mint_observation(
    out: &UncheckedAccount,
    tag: u8,
    supply: u64,
    decimals: u8,
    is_initialized: bool,
    mint_authority: Option<[u8; 32]>,
    freeze_authority: Option<[u8; 32]>,
) -> Result<()> {
    let mut out_view = out.account().clone();
    let data = unsafe { out_view.borrow_unchecked_mut() };
    clear(data);
    data[0] = tag;
    data[1..9].copy_from_slice(&supply.to_le_bytes());
    data[9] = decimals;
    data[10] = is_initialized as u8;
    write_option_key(data, 11, mint_authority);
    write_option_key(data, 44, freeze_authority);
    Ok(())
}

fn write_token_account_observation(
    out: &UncheckedAccount,
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
    let mut out_view = out.account().clone();
    let data = unsafe { out_view.borrow_unchecked_mut() };
    clear(data);
    data[0] = tag;
    data[1..33].copy_from_slice(&mint);
    data[33..65].copy_from_slice(&owner);
    data[65..73].copy_from_slice(&amount.to_le_bytes());
    write_option_key(data, 73, delegate);
    data[106] = state;
    write_option_u64(data, 107, is_native);
    data[116..124].copy_from_slice(&delegated_amount.to_le_bytes());
    write_option_key(data, 124, close_authority);
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
