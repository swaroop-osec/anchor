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

    // Generate validation for wrapper types (HasOne, Owned, Executable, etc.)
    let wrapper_validations: Vec<proc_macro2::TokenStream> = non_init_fields
        .iter()
        .filter_map(|af| match af {
            AccountField::Field(f) => generate_wrapper_validation(f, accs),
            AccountField::CompositeField(_) => None,
        })
        .collect();

    quote! {
        #duplicate_checks
        #(#access_checks)*
        #(#wrapper_validations)*
    }
}

/// Generate validation code for wrapper types like HasOne<T, Target>.
fn generate_wrapper_validation(
    f: &Field,
    accs: &AccountsStruct,
) -> Option<proc_macro2::TokenStream> {
    // Check if this field uses a HasOne wrapper (possibly nested)
    if let Some(has_one_ty) = extract_has_one_ty(&f.ty) {
        return Some(generate_has_one_wrapper_validation(f, has_one_ty, accs));
    }
    // Future: Add validation for other wrapper types as needed
    None
}

/// Recursively extract HasOneTy from a type (handles nested wrappers like Mut<HasOne<...>>)
fn extract_has_one_ty(ty: &Ty) -> Option<&crate::HasOneTy> {
    match ty {
        Ty::HasOne(has_one_ty) => Some(has_one_ty),
        Ty::Mut(mut_ty) => extract_has_one_ty(&mut_ty.inner),
        Ty::Seeded(seeded_ty) => extract_has_one_ty(&seeded_ty.inner),
        Ty::Owned(owned_ty) => extract_has_one_ty(&owned_ty.inner),
        Ty::Executable(exec_ty) => extract_has_one_ty(&exec_ty.inner),
        _ => None,
    }
}

/// Generate HasOne validation code.
/// Uses `HasOneTarget::FIELD` to locate the target account and validate.
fn generate_has_one_wrapper_validation(
    f: &Field,
    has_one_ty: &crate::HasOneTy,
    accs: &AccountsStruct,
) -> proc_macro2::TokenStream {
    let field_ident = &f.ident;
    let target_type = &has_one_ty.target_type_path;
    let target_account_ty = f.account_ty();

    let target_field_names: Vec<String> = accs
        .fields
        .iter()
        .filter_map(|af| match af {
            AccountField::Field(field) => Some(field.ident.to_string()),
            AccountField::CompositeField(_) => None,
        })
        .collect();
    let target_field_checks: Vec<proc_macro2::TokenStream> = target_field_names
        .iter()
        .map(|name| {
            let lit = proc_macro2::Literal::string(name);
            quote! { __anchor_str_eq(__target_field, #lit) }
        })
        .collect();
    let target_fields_list = if target_field_names.is_empty() {
        "<none>".to_string()
    } else {
        target_field_names.join(", ")
    };
    let target_type_str = quote::quote!(#target_type).to_string().replace(' ', "");
    let field_name = field_ident.to_string();
    let error_msg = format!(
        "HasOne<_, {}> on field '{}' expects HasOneTarget::FIELD to match one of the top-level accounts fields: {}",
        target_type_str,
        field_name,
        target_fields_list
    );
    let error_literal = proc_macro2::Literal::string(&error_msg);

    let target_key_arms: Vec<proc_macro2::TokenStream> = accs
        .fields
        .iter()
        .filter_map(|af| match af {
            AccountField::Field(field) => {
                let target_ident = &field.ident;
                let target_name = target_ident.to_string();
                let target_key = if field.is_optional {
                    quote! {
                        let target_account = if let Some(account) = #target_ident {
                            account
                        } else {
                            return Err(anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintAccountIsNone
                            ).with_account_name(#target_name));
                        };
                        anchor_lang::Key::key(target_account)
                    }
                } else {
                    quote! { anchor_lang::Key::key(#target_ident) }
                };
                Some(quote! { #target_name => { #target_key } })
            }
            AccountField::CompositeField(_) => None,
        })
        .collect();

    let const_check = if target_field_checks.is_empty() {
        quote! {
            const _: () = {
                panic!(#error_literal);
            };
        }
    } else {
        quote! {
            const _: () = {
                const fn __anchor_str_eq(a: &str, b: &str) -> bool {
                    let a_bytes = a.as_bytes();
                    let b_bytes = b.as_bytes();
                    if a_bytes.len() != b_bytes.len() {
                        return false;
                    }
                    let mut i = 0;
                    while i < a_bytes.len() {
                        if a_bytes[i] != b_bytes[i] {
                            return false;
                        }
                        i += 1;
                    }
                    true
                }

                let __target_field = <#target_type as anchor_lang::account_set::HasOneTarget<#target_account_ty>>::FIELD;
                if !(#(#target_field_checks)||*) {
                    panic!(#error_literal);
                }
            };
        }
    };

    let validate_block = quote! {
        #const_check
        let __target_key = match <#target_type as anchor_lang::account_set::HasOneTarget<#target_account_ty>>::FIELD {
            #(#target_key_arms,)*
            _ => {
                unreachable!(#error_literal);
            }
        };

        if let Err(err) = #field_ident.validate_has_one(&__target_key) {
            return Err(err.with_account_name(stringify!(#field_ident)));
        }
    };

    if f.is_optional {
        quote! {
            // HasOne wrapper validation for field '#field_ident'
            if let Some(#field_ident) = #field_ident {
                #validate_block
            }
        }
    } else {
        quote! {
            // HasOne wrapper validation for field '#field_ident'
            #validate_block
        }
    }
}

// Init constraints are handled during account deserialization only.
fn is_init(af: &AccountField) -> bool {
    match af {
        AccountField::CompositeField(_) => false,
        AccountField::Field(f) => f.constraints.init.is_some(),
    }
}
