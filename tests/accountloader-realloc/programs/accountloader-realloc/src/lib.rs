use anchor_lang::prelude::*;

declare_id!("8GM8KqKaxYb1jEbn5TiqqPqJYsChhjYvfpT2f6KTmUKb");

#[program]
pub mod accountloader_realloc {
    use super::*;

    /// Create the zero-copy account with the full required footprint
    /// (`DISCRIMINATOR.len() + size_of::<Data>()`).
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let mut data = ctx.accounts.data.load_init()?;
        data.value = 42;
        Ok(())
    }

    /// Shrink the `AccountLoader` to `new_len` bytes
    pub fn shrink(_ctx: Context<Shrink>, _new_len: u16) -> Result<()> {
        Ok(())
    }

    /// Verify the account stays readable after the shrink.
    pub fn read(ctx: Context<Read>) -> Result<u64> {
        let data = ctx.accounts.data.load()?;
        Ok(data.value)
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        seeds = [b"data"],
        bump,
        space = 8 + core::mem::size_of::<Data>(),
    )]
    pub data: AccountLoader<'info, Data>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(new_len: u16)]
pub struct Shrink<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"data"],
        bump,
        realloc = new_len as usize,
        realloc::payer = authority,
        realloc::zero = false,
    )]
    pub data: AccountLoader<'info, Data>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Read<'info> {
    #[account(seeds = [b"data"], bump)]
    pub data: AccountLoader<'info, Data>,
}

#[account(zero_copy)]
#[repr(C)]
pub struct Data {
    pub value: u64,
    pub padding: [u8; 64],
}
