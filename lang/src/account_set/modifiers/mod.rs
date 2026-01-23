//! Constraint wrapper types (modifiers) for composable validation.
//!
//! These wrappers enforce specific constraints and can be nested to combine
//! multiple constraints on a single account.
//!
//! # Available Wrappers
//!
//! ## Validation Wrappers
//! - [`Mut<T>`]: Enforces account is writable
//! - [`Seeded<T, S>`]: Validates PDA derivation and captures bump
//! - [`Owned<T, P>`]: Validates account is owned by a specific program
//! - [`Executable<T>`]: Validates account is an executable program
//! - [`HasOne<T, Target>`]: Validates a relationship between accounts
//!
//! Note: Init/close/realloc use `#[account(...)]` attribute syntax.
//!
//! # Composition
//!
//! Wrappers are designed to be composed:
//!
//! ```ignore
//! // Account must be writable
//! Mut<Account<'info, MyData>>
//!
//! // Account must be a PDA with specific seeds
//! Seeded<Account<'info, MyData>, MySeeds>
//!
//! // Account must be a PDA with specific seeds AND writable
//! Mut<Seeded<Account<'info, MyData>, MySeeds>>
//!
//! // Account must be owned by a specific program
//! Owned<UncheckedAccount<'info>, TokenProgram>
//!
//! // Account must be executable (is a program)
//! Executable<UncheckedAccount<'info>>
//! ```

mod executable;
mod has_one;
mod owned;
mod seeded;
mod writable;

pub use executable::Executable;
pub use has_one::{HasOne, HasOneTarget};
pub use owned::Owned;
pub use seeded::{Seeded, Seeds, SeedsWithBump};
pub use writable::Mut;
