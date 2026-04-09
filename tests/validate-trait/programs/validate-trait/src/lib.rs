use anchor_lang::prelude::*;

declare_id!("AtLDjzwcEQVEKzM5uMCZXQkshumS2ksuurw3tBmro5Ek");

#[program]
pub mod validate_trait {
    use super::*;

    pub fn set_data(ctx: Context<SetData>, amount: u64) -> Result<()> {
        ctx.accounts.my_account.data = amount;
        ctx.accounts.my_account.authority = ctx.accounts.authority.key();
        Ok(())
    }

    pub fn manual_set_data(ctx: Context<ManualSetData>, amount: u64) -> Result<()> {
        ctx.accounts.my_account.data = amount;
        Ok(())
    }

    pub fn custom_struct_set_data(
        ctx: Context<CustomStructSetData>,
        args: MyCustomArgs,
    ) -> Result<()> {
        ctx.accounts.my_account.data = args.amount;
        Ok(())
    }

    pub fn set_balance(ctx: Context<SetBalance>, amount: u64) -> Result<()> {
        ctx.accounts.my_account.data = amount;
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct MyCustomArgs {
    pub amount: u64,
}

#[account]
#[derive(Default)]
pub struct MyAccount {
    pub data: u64,
    pub authority: Pubkey,
}

#[derive(Accounts)]
#[instruction(amount: u64)]
#[validate]
pub struct SetData<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 8 + 32,
        constraint = amount > 10
    )]
    pub my_account: Account<'info, MyAccount>,
    #[account(mut, constraint = *amount < 1000)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct ManualSetData<'info> {
    #[account(mut)]
    pub my_account: Account<'info, MyAccount>,
    pub authority: Signer<'info>,
}

// Manually implement Validate for ManualSetData
impl<'info> Validate for ManualSetData<'info> {
    type IxArgs = ManualSetDataArgs;

    fn validate<'ctx_info>(
        &self,
        _ctx: &Context<'ctx_info, Self>,
        args: &Self::IxArgs,
    ) -> Result<()> {
        if args.amount == 42 {
            return err!(ErrorCode::InstructionDidNotDeserialize);
        }
        if self.my_account.authority != self.authority.key() {
            return err!(ErrorCode::ConstraintHasOne);
        }
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(args: MyCustomArgs)]
pub struct CustomStructSetData<'info> {
    #[account(mut)]
    pub my_account: Account<'info, MyAccount>,
    pub authority: Signer<'info>,
}

// Use the custom struct via the generated wrapper
impl<'info> Validate for CustomStructSetData<'info> {
    type IxArgs = CustomStructSetDataArgs;

    fn validate<'ctx_info>(
        &self,
        _ctx: &Context<'ctx_info, Self>,
        ix_args: &Self::IxArgs,
    ) -> Result<()> {
        if ix_args.args.amount == 666 {
            return err!(ErrorCode::InstructionDidNotDeserialize);
        }
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(amount: u64)]
#[validate]
pub struct SetBalance<'info> {
    #[account(
        mut, 
        constraint = my_account.data > *amount
    )]
    pub my_account: Account<'info, MyAccount>,
}
