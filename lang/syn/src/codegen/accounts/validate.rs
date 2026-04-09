use {
    crate::{
        codegen::accounts::{
            constraints, generics, try_accounts::generate_duplicate_mutable_checks,
        },
        AccountField, AccountsStruct,
    },
    quote::quote,
    syn::Expr,
};

/// Generates the `Validate` implementation for an accounts struct.
///
/// When `#[validate]` is present (`derive_constraints = true`), emits a full
/// impl that runs all non-mutating constraint checks (access guards, `has_one`,
/// `constraint = …`, `owner`, `address`, `signer`, token/mint checks, etc.) as
/// well as duplicate-mutable-account detection.
///
/// Constraints that mutate account state (`init`, `zeroed`, `realloc`) are
/// intentionally left in `try_accounts` only — they must not run twice.
///
/// Without `#[validate]`, no impl is emitted and the user must provide their
/// own `Validate` impl with custom cross-field validation logic.
pub fn generate(accs: &AccountsStruct) -> proc_macro2::TokenStream {
    if !accs.derive_constraints {
        return quote! {};
    }
    let name = &accs.ident;
    let args_name = syn::Ident::new(&format!("{}Args", name), name.span());

    let crate::codegen::accounts::ParsedGenerics {
        combined_generics,
        trait_generics: _,
        struct_generics,
        where_clause,
    } = generics(accs);

    // Create destructuring patterns for accounts and arguments.
    // This allows bare identifiers in constraints to work while keeping
    // the expanded code clean and readable.
    let field_idents: Vec<proc_macro2::TokenStream> = accs
        .fields
        .iter()
        .map(|af| {
            let id = match af {
                AccountField::CompositeField(s) => &s.ident,
                AccountField::Field(f) => &f.ident,
            };
            quote! { #id }
        })
        .collect();

    let arg_idents: Vec<proc_macro2::TokenStream> = match &accs.instruction_api {
        Some(ix_api) => ix_api
            .iter()
            .map(|expr| {
                if let Expr::Type(expr_type) = expr {
                    let arg_name = &expr_type.expr;
                    quote! { #arg_name }
                } else {
                    panic!("Invalid instruction declaration");
                }
            })
            .collect(),
        None => vec![],
    };

    let accounts_destructure = if field_idents.is_empty() {
        quote! {}
    } else {
        quote! { let Self { #(#field_idents),*, .. } = self; }
    };

    let args_destructure = if arg_idents.is_empty() {
        quote! {}
    } else {
        quote! { let Self::IxArgs { #(#arg_idents),*, .. } = args; }
    };

    // Run constraints for fields that do NOT involve account-state mutations.
    // `init`, `zeroed`, and `realloc` allocate/write/resize accounts and must
    // stay exclusively in `try_accounts`.
    //
    // `generate_for_validate` is used instead of `generate`; it is identical
    // except that it also runs raw `constraint = <expr>` checks.
    let access_checks: Vec<proc_macro2::TokenStream> = accs
        .fields
        .iter()
        .filter_map(|af| match af {
            AccountField::Field(f) => {
                if f.constraints.init.is_some()
                    || f.constraints.zeroed.is_some()
                    || f.constraints.realloc.is_some()
                {
                    None
                } else {
                    Some(constraints::generate_for_validate(f, accs))
                }
            }
            AccountField::CompositeField(s) => {
                // TODO(composite-validate): Recursively call `Validate::validate` on
                // this nested accounts struct so its own constraint checks run as
                // part of the outer struct's validation phase.
                //
                // For now, only the raw `#[account(constraint = ...)]` expressions
                // on the composite field itself are checked via generate_composite.
                Some(constraints::generate_composite(s))
            }
        })
        .collect();

    // Duplicate mutable account check (same logic as in try_accounts).
    let duplicate_checks = generate_duplicate_mutable_checks(accs);

    quote! {
        #[automatically_derived]
        impl<#combined_generics> anchor_lang::validate::Validate
            for #name<#struct_generics>
        #where_clause
        {
            type IxArgs = #args_name;

            fn validate<'__info>(
                &self,
                _ctx: &anchor_lang::context::Context<'__info, Self>,
                args: &Self::IxArgs,
            ) -> anchor_lang::Result<()> {
                // Bind identifiers so generated constraint code can reference
                // them by name (mirroring try_accounts locals).
                #accounts_destructure
                #args_destructure

                // Variables the generated constraint snippets may reference.
                let __program_id = _ctx.program_id;
                #[allow(unused_mut)]
                let mut __bumps = <Self as anchor_lang::Bumps>::Bumps::default();
                #[allow(unused_mut)]
                let mut __reallocs = std::collections::BTreeSet::<
                    anchor_lang::solana_program::pubkey::Pubkey,
                >::new();

                // Duplicate mutable account check.
                #duplicate_checks

                // Per-field access constraint checks.
                #(#access_checks)*

                Ok(())
            }
        }
    }
}
