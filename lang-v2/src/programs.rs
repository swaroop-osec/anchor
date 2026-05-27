//! Well-known program marker types for use with `Program<T>`.
//!
//! IDs are const-evaluated via `address!` (base58 decoded at compile time).
//! Each marker exposes `IDL_ADDRESS`, forwarded through `IdlAccountType::__IDL_ADDRESS`
//! at emission time.
//!
//! The runtime ID returned by each marker is an `Address`, not a separate
//! legacy key type, so callers can pass `Token::id()` directly into modern
//! Solana instruction/account-meta builders.

use {crate::Id, pinocchio::address::Address};

pub struct System;
impl Id for System {
    fn id() -> Address {
        const ADDR: Address = crate::address!("11111111111111111111111111111111");
        ADDR
    }
    const IDL_ADDRESS: &'static str = "11111111111111111111111111111111";
}

pub struct Token;
impl Id for Token {
    fn id() -> Address {
        const ADDR: Address = crate::address!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
        ADDR
    }
    const IDL_ADDRESS: &'static str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
}

pub struct Token2022;
impl Id for Token2022 {
    fn id() -> Address {
        const ADDR: Address = crate::address!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
        ADDR
    }
    const IDL_ADDRESS: &'static str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
}

pub struct AssociatedToken;
impl Id for AssociatedToken {
    fn id() -> Address {
        const ADDR: Address = crate::address!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
        ADDR
    }
    const IDL_ADDRESS: &'static str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
}

pub struct Memo;
impl Id for Memo {
    fn id() -> Address {
        const ADDR: Address = crate::address!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");
        ADDR
    }
    const IDL_ADDRESS: &'static str = "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr";
}
