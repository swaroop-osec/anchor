//! Probe v2's instruction-arg deserializer for trailing-byte strictness.
//!
//! Original buggy site — `lang-v2/derive/src/lib.rs:147-169`:
//!
//! ```rust
//! let error_handling = if inline_error {
//!     quote! {
//!         match anchor_lang_v2::wincode::config::deserialize(
//!             __ix_data,
//!             anchor_lang_v2::BORSH_CONFIG,
//!         ) {
//!             Ok(__v) => __v,
//!             Err(_) => return {
//!                 let __e: anchor_lang_v2::Error =
//!                     anchor_lang_v2::ErrorCode::InstructionDidNotDeserialize.into();
//!                 __e.into()
//!             },
//!         }
//!     }
//! } else {
//!     quote! {
//!         anchor_lang_v2::wincode::config::deserialize(
//!             __ix_data,
//!             anchor_lang_v2::BORSH_CONFIG,
//!         )
//!             .map_err(|_| anchor_lang_v2::ErrorCode::InstructionDidNotDeserialize)?
//!     }
//! };
//! ```
//!
//! Suggested fix — insert after the `let __args: ...` line (lib.rs:173):
//!
//! ```rust
//! let consumed = anchor_lang_v2::wincode::config::deserialized_size(&__args, BORSH_CONFIG)?;
//! if consumed != __ix_data.len() {
//!     return Err(anchor_lang_v2::ErrorCode::InstructionDidNotDeserialize.into());
//! }
//! ```

use anchor_lang_v2::prelude::*;

declare_id!("ArgFuzz1111111111111111111111111111111111111");

#[program]
pub mod arg_fuzz {
    use super::*;

    pub fn book_flag_check(_ctx: &mut Context<NoAccounts>, flag: bool) -> Result<()> {
        msg!("book_flag_check ran flag={}", flag);
        Ok(())
    }

    pub fn one_u64(_ctx: &mut Context<NoAccounts>, a: u64) -> Result<()> {
        msg!("one_u64 ran a={}", a);
        Ok(())
    }

    pub fn no_args(_ctx: &mut Context<NoAccounts>) -> Result<()> {
        msg!("no_args ran");
        Ok(())
    }

    pub fn two_u64(_ctx: &mut Context<NoAccounts>, a: u64, b: u64) -> Result<()> {
        msg!("two_u64 ran a={} b={}", a, b);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct NoAccounts {}
