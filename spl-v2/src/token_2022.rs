//! Token-2022 CPI helpers that are not part of the legacy Token program.

pub mod cpi {
    extern crate alloc;

    use {
        alloc::{vec, vec::Vec},
        anchor_lang_v2::{CpiContext, CpiHandle, ToCpiAccounts},
        pinocchio::instruction::InstructionAccount,
    };

    pub use pinocchio_token_2022::instructions::ExtensionDiscriminator;

    pub mod accounts {
        use super::*;

        pub struct CreateNativeMint<'a> {
            pub payer: CpiHandle<'a>,
            pub native_mint: CpiHandle<'a>,
            pub system_program: CpiHandle<'a>,
        }

        impl<'a> ToCpiAccounts<'a> for CreateNativeMint<'a> {
            fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
                vec![
                    InstructionAccount::writable_signer(self.payer.address()),
                    InstructionAccount::writable(self.native_mint.address()),
                    InstructionAccount::new(self.system_program.address(), false, false),
                ]
            }

            fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
                vec![self.payer, self.native_mint, self.system_program]
            }
        }

        pub struct InitializeNonTransferableMint<'a> {
            pub mint: CpiHandle<'a>,
        }

        impl<'a> ToCpiAccounts<'a> for InitializeNonTransferableMint<'a> {
            fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
                vec![InstructionAccount::writable(self.mint.address())]
            }

            fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
                vec![self.mint]
            }
        }

        pub struct Reallocate<'a> {
            pub account: CpiHandle<'a>,
            pub payer: CpiHandle<'a>,
            pub system_program: CpiHandle<'a>,
            pub owner: CpiHandle<'a>,
        }

        impl<'a> ToCpiAccounts<'a> for Reallocate<'a> {
            fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
                vec![
                    InstructionAccount::writable(self.account.address()),
                    InstructionAccount::writable_signer(self.payer.address()),
                    InstructionAccount::new(self.system_program.address(), false, false),
                    InstructionAccount::readonly_signer(self.owner.address()),
                ]
            }

            fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
                vec![self.account, self.payer, self.system_program, self.owner]
            }
        }

        pub struct WithdrawExcessLamports<'a> {
            pub source: CpiHandle<'a>,
            pub destination: CpiHandle<'a>,
            pub authority: CpiHandle<'a>,
        }

        impl<'a> ToCpiAccounts<'a> for WithdrawExcessLamports<'a> {
            fn to_instruction_accounts(&self) -> Vec<InstructionAccount<'a>> {
                vec![
                    InstructionAccount::writable(self.source.address()),
                    InstructionAccount::writable(self.destination.address()),
                    InstructionAccount::readonly_signer(self.authority.address()),
                ]
            }

            fn to_cpi_handles(&self) -> Vec<CpiHandle<'a>> {
                vec![self.source, self.destination, self.authority]
            }
        }
    }

    const DISC_REALLOCATE: u8 = 29;
    const DISC_CREATE_NATIVE_MINT: u8 = 31;
    const DISC_INITIALIZE_NON_TRANSFERABLE_MINT: u8 = 32;
    const DISC_WITHDRAW_EXCESS_LAMPORTS: u8 = 38;

    fn encode_reallocate_ix(extension_types: &[ExtensionDiscriminator]) -> Vec<u8> {
        let mut data = Vec::with_capacity(1 + extension_types.len() * 2);
        data.push(DISC_REALLOCATE);
        for extension_type in extension_types {
            data.extend_from_slice(&(*extension_type as u16).to_le_bytes());
        }
        data
    }

    pub fn create_native_mint<'a>(ctx: CpiContext<'a, accounts::CreateNativeMint<'a>>) {
        ctx.invoke(&[DISC_CREATE_NATIVE_MINT]);
    }

    pub fn initialize_non_transferable_mint<'a>(
        ctx: CpiContext<'a, accounts::InitializeNonTransferableMint<'a>>,
    ) {
        ctx.invoke(&[DISC_INITIALIZE_NON_TRANSFERABLE_MINT]);
    }

    pub fn reallocate<'a>(
        ctx: CpiContext<'a, accounts::Reallocate<'a>>,
        extension_types: &[ExtensionDiscriminator],
    ) {
        ctx.invoke(&encode_reallocate_ix(extension_types));
    }

    pub fn withdraw_excess_lamports<'a>(ctx: CpiContext<'a, accounts::WithdrawExcessLamports<'a>>) {
        ctx.invoke(&[DISC_WITHDRAW_EXCESS_LAMPORTS]);
    }
}
