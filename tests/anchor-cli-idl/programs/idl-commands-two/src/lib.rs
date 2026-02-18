use anchor_lang::prelude::*;

declare_id!("2uA3amp95zsEHUpo8qnLMhcFAUsiKVEcKHXS1JetFjU5");

#[program]
pub mod idl_commands_two {
    use super::*;

    pub fn uninitialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
