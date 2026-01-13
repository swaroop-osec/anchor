use crate::codegen::accounts::{bumps, constraints, generics, ParsedGenerics};
use crate::{AccountField, AccountTy, AccountsStruct, Field, InterfaceAccountTy, Ty};
use quote::quote;

pub fn generate(accs: &AccountsStruct) -> proc_macro2::TokenStream {
    // Emit a Constraints impl that reuses account attribute validation.
    let name = &accs.ident;
    let ParsedGenerics {
        combined_generics,
        trait_generics: _,
        struct_generics,
        where_clause,
    } = generics(accs);
    let bumps_struct_name = bumps::generate_bumps_name(&accs.ident);

    let field_aliases: Vec<proc_macro2::TokenStream> =
        accs.fields.iter().map(generate_field_alias).collect();

    let constraints = generate_validate_constraints(accs);

    quote! {
        #[automatically_derived]
        impl<#combined_generics> anchor_lang::constraints::Constraints for #name<#struct_generics> #where_clause {
            fn validate<'__info>(
                &self,
                _ctx: &anchor_lang::context::Context<'_, '_, '_, '__info, Self>,
            ) -> anchor_lang::Result<()> {
                #[allow(unused_variables)]
                let __program_id = _ctx.program_id;
                let __accounts = _ctx.remaining_accounts;
                #[allow(unused_variables, unused_mut)]
                let mut __bumps = #bumps_struct_name::default();
                #(#field_aliases)*
                #constraints
                Ok(())
            }
        }
    }
}

// Bind account fields to locals that match constraint generator expectations.
fn generate_field_alias(af: &AccountField) -> proc_macro2::TokenStream {
    match af {
        AccountField::CompositeField(s) => {
            let ident = &s.ident;
            quote! {
                let #ident = &self.#ident;
            }
        }
        AccountField::Field(f) => {
            let ident = &f.ident;
            let accessor = field_accessor(f);
            quote! {
                let #ident = #accessor;
            }
        }
    }
}

// Normalize access to boxed/optional fields for constraint checks.
fn field_accessor(f: &Field) -> proc_macro2::TokenStream {
    let ident = &f.ident;
    let is_boxed = matches!(
        f.ty,
        Ty::Account(AccountTy { boxed: true, .. })
            | Ty::InterfaceAccount(InterfaceAccountTy { boxed: true, .. })
    );

    match (is_boxed, f.is_optional) {
        (true, true) => quote! { self.#ident.as_deref() },
        (true, false) => quote! { self.#ident.as_ref() },
        (false, true) => quote! { self.#ident.as_ref() },
        (false, false) => quote! { &self.#ident },
    }
}

// Reuse constraint validation without init/zeroed/realloc side effects.
fn generate_validate_constraints(accs: &AccountsStruct) -> proc_macro2::TokenStream {
    let non_init_fields: Vec<&AccountField> =
        accs.fields.iter().filter(|af| !is_init(af)).collect();

    let duplicate_checks = super::try_accounts::generate_duplicate_mutable_checks(accs);

    let access_checks: Vec<proc_macro2::TokenStream> = non_init_fields
        .iter()
        .map(|af: &&AccountField| match af {
            AccountField::Field(f) => constraints::generate_for_validate(f, accs),
            AccountField::CompositeField(s) => constraints::generate_composite(s),
        })
        .collect();

    quote! {
        #duplicate_checks
        #(#access_checks)*
    }
}

// Init constraints are handled during account deserialization only.
fn is_init(af: &AccountField) -> bool {
    match af {
        AccountField::CompositeField(_) => false,
        AccountField::Field(f) => f.constraints.init.is_some(),
    }
}
