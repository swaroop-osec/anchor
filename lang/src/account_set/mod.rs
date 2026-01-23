//! Account set traits and composable constraint wrappers.
//!
//! This module provides a trait-based approach to account validation, inspired by Star Frame.
//! Instead of macro-generated constraint checks, validation logic lives in runtime traits
//! and composable wrapper types.
//!
//! # Core Concepts
//!
//! - **SingleAccountSet**: A trait for types that represent a single account with basic checks
//!
//! # Constraint Wrappers
//!
//! ## Validation Wrappers
//! - [`Mut<T>`](modifiers::Mut): Enforces `is_writable`
//! - [`Seeded<T, S>`](modifiers::Seeded): Validates PDA and captures bump
//! - [`Owned<T, P>`](modifiers::Owned): Validates account owner
//! - [`Executable<T>`](modifiers::Executable): Validates account is a program
//! - [`HasOne<T, Target>`](modifiers::HasOne): Validates account relationships
//!
//! Note: Account initialization uses `#[account(init, ...)]` attribute syntax.
//!
//! # Composition
//!
//! Wrappers can be composed to build constraint chains:
//!
//! ```ignore
//! // Mut<Seeded<Account<'info, MyData>, MySeeds>> enforces writable AND PDA validation
//! pub struct MyAccounts<'info> {
//!     pub data: Mut<Seeded<Account<'info, MyData>, MySeeds>>,
//! }
//! ```
//!
//! # Migration from Macro Constraints
//!
//! | Old syntax | New wrapper |
//! |------------|-------------|
//! | `#[account(mut)]` | `Mut<T>` |
//! | `#[account(seeds = [...], bump)]` | `Seeded<T, S>` |
//! | `#[account(owner = ...)]` | `Owned<T, P>` |
//! | `#[account(executable)]` | `Executable<T>` |
//! | `#[account(has_one = ...)]` | `HasOne<T, Target>` |
//! Note: For signer validation, use the existing `Signer<'info>` type.
//! Note: For init/close/realloc, use the attribute syntax (e.g., `#[account(init, ...)]`).

mod impls;
pub mod modifiers;
mod single_set;

pub use modifiers::{Executable, HasOne, HasOneTarget, Mut, Owned, Seeded, Seeds, SeedsWithBump};
pub use single_set::SingleAccountSet;
