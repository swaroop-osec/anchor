use crate::prelude::*;
use crate::Bumps;

/// Trait for the **validation** phase of the account lifecycle, which runs
/// after deserialization and before the user handler:
///
/// 1. **Deserialize** — `Accounts::try_accounts` (account loading + `init`/`realloc`)
/// 2. **Validate**    — `Validate::validate` (this trait — constraint checks)
/// 3. **Cleanup**     — `AccountsExit::exit`
///
/// ## Auto-generated implementation (`#[validate]`)
///
/// Adding `#[validate]` to a `#[derive(Accounts)]` struct generates a full
/// `Validate` implementation that runs all account attribute constraint checks
/// (`mut`, `has_one`, `owner`, `signer`, `address`, token/mint guards,
/// duplicate-mutable detection, etc.).
///
/// ```rust,ignore
/// #[derive(Accounts)]
/// #[instruction(amount: u64)]
/// #[validate]                        // ← generates the Validate impl
/// pub struct Transfer<'info> {
///     #[account(mut, has_one = authority)]
///     pub vault: Account<'info, Vault>,
///     pub authority: Signer<'info>,
/// }
/// ```
///
/// ## Manual implementation (no `#[validate]`)
///
/// If `#[validate]` is omitted, **no implementation is generated** and you
/// must provide one yourself. You are responsible for **all** validation —
/// attribute constraints on the struct (`mut`, `has_one`, `owner`, `signer`,
/// etc.) are **not** enforced automatically.
///
/// > **Note on `Signer<'info>`**: fields typed as `Signer<'info>` are an
/// > exception — the type's own `try_accounts` checks `is_signer`, so that
/// > requirement is still enforced even without `#[validate]`.
/// >
/// > **Warning**: the following are **not** enforced in a manual impl:
/// > - `#[account(mut)]` / `#[account(signer)]` on non-`Signer` types —
/// >   you must check `account.to_account_info().is_writable` /
/// >   `account.is_signer` yourself.
/// > - **Duplicate mutable account detection** — the protection against
/// >   passing the same writable account in two fields is only generated
/// >   when `#[validate]` is present. A manual impl must replicate this
/// >   check or be designed so aliasing is impossible.
///
/// ```rust,ignore
/// #[derive(Accounts)]
/// #[instruction(amount: u64)]
/// pub struct Transfer<'info> {
///     // `#[account(mut)]` sets IDL metadata so the client marks the account
///     // writable, but without `#[validate]` the program does NOT enforce it
///     // automatically — the manual impl below must do so explicitly.
///     #[account(mut)]
///     pub from: Account<'info, TokenAccount>,
///     #[account(mut)]
///     pub to: Account<'info, TokenAccount>,
///     // `Signer<'info>` is an exception: its `try_accounts` always checks
///     // `is_signer`, so this field is enforced even without `#[validate]`.
///     pub authority: Signer<'info>,
/// }
///
/// impl<'info> Validate for Transfer<'info> {
///     type Args = TransferArgs; // generated from #[instruction(amount: u64)]
///
///     fn validate(&self, _ctx: &Context<'info, Self>, args: &Self::Args) -> Result<()> {
///         // Writability — not auto-checked without #[validate].
///         require!(self.from.to_account_info().is_writable, ErrorCode::ConstraintMut);
///         require!(self.to.to_account_info().is_writable, ErrorCode::ConstraintMut);
///         // Cross-field / business-logic checks.
///         require!(self.from.owner == self.authority.key(), ErrorCode::ConstraintHasOne);
///         require!(self.from.key() != self.to.key(), MyError::SameSourceAndDestination);
///         require!(self.from.amount >= args.amount, MyError::InsufficientBalance);
///         Ok(())
///     }
/// }
/// ```
///
/// Automatically called via [`Context::validate`] inside the generated
/// instruction dispatch, before the user handler runs.
#[diagnostic::on_unimplemented(
    message = "`{Self}` must implement `Validate` for account validation",
    label = "`Validate` is required here",
    note = "Add `#[validate]` to your `#[derive(Accounts)]` struct to automatically\n\
            generate a `Validate` impl that runs all account attribute constraints.\n\
            \n\
            Alternatively, implement the trait manually for custom validation:\n\
            \n\
            impl<'info> Validate for MyAccounts<'info> {{\n\
                type IxArgs = MyAccountsArgs;\n\
                \n\
                fn validate(&self, ctx: &Context<'info, Self>, args: &Self::IxArgs) -> Result<()> {{\n\
                    // business-logic validation here\n\
                    Ok(())\n\
                }}\n\
            }}"
)]
pub trait Validate: Bumps + Sized {
    type IxArgs: AnchorDeserialize;
    fn validate<'info>(&self, ctx: &Context<'info, Self>, args: &Self::IxArgs) -> Result<()>;
}
