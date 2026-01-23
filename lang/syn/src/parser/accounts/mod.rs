pub mod constraints;
#[cfg(feature = "event-cpi")]
pub mod event_cpi;

use crate::parser::docs;
use crate::*;
use syn::parse::{Error as ParseError, Result as ParseResult};
use syn::Attribute;
use syn::Path;

pub fn parse(accounts_struct: &syn::ItemStruct) -> ParseResult<AccountsStruct> {
    let instruction_api: Option<Punctuated<Expr, Comma>> = accounts_struct
        .attrs
        .iter()
        .find(|a| {
            a.path
                .get_ident()
                .is_some_and(|ident| ident == "instruction")
        })
        .map(|ix_attr| ix_attr.parse_args_with(Punctuated::<Expr, Comma>::parse_terminated))
        .transpose()?;

    #[cfg(feature = "event-cpi")]
    let accounts_struct = {
        let is_event_cpi = accounts_struct
            .attrs
            .iter()
            .filter_map(|attr| attr.path.get_ident())
            .any(|ident| *ident == "event_cpi");
        if is_event_cpi {
            event_cpi::add_event_cpi_accounts(accounts_struct)?
        } else {
            accounts_struct.clone()
        }
    };
    #[cfg(not(feature = "event-cpi"))]
    let accounts_struct = accounts_struct.clone();

    let manual_constraints = parse_manual_constraints(&accounts_struct.attrs)?;

    let fields = match &accounts_struct.fields {
        syn::Fields::Named(fields) => fields
            .named
            .iter()
            .map(parse_account_field)
            .collect::<ParseResult<Vec<AccountField>>>()?,
        _ => {
            return Err(ParseError::new_spanned(
                &accounts_struct.fields,
                "fields must be named",
            ))
        }
    };

    constraints_cross_checks(&fields)?;

    Ok(AccountsStruct::new(
        accounts_struct,
        fields,
        instruction_api,
        manual_constraints,
    ))
}

fn parse_manual_constraints(attrs: &[Attribute]) -> ParseResult<bool> {
    // Parse #[accounts(manual_constraints)] to skip auto Constraints impls.
    let mut manual_constraints = false;

    for attr in attrs.iter().filter(|a| a.path.is_ident("accounts")) {
        let args = attr.parse_args_with(Punctuated::<Ident, Comma>::parse_terminated)?;
        for arg in args {
            if arg == "manual_constraints" {
                manual_constraints = true;
            } else {
                return Err(ParseError::new_spanned(arg, "unknown #[accounts] argument"));
            }
        }
    }

    Ok(manual_constraints)
}

fn constraints_cross_checks(fields: &[AccountField]) -> ParseResult<()> {
    // COMMON ERROR MESSAGE
    let message = |constraint: &str, field: &str, required: bool| {
        if required {
            format! {
                "a non-optional {constraint} constraint requires \
                a non-optional {field} field to exist in the account \
                validation struct. Use the Program type to add \
                the {field} field to your validation struct."
            }
        } else {
            format! {
                "an optional {constraint} constraint requires \
                an optional or required {field} field to exist \
                in the account validation struct. Use the Program type \
                to add the {field} field to your validation struct."
            }
        }
    };

    // INIT
    let mut required_init = false;
    let init_fields: Vec<&Field> = fields
        .iter()
        .filter_map(|f| match f {
            AccountField::Field(field) if field.constraints.init.is_some() => {
                if !field.is_optional {
                    required_init = true
                }
                Some(field)
            }
            _ => None,
        })
        .collect();

    for field in &init_fields {
        if matches!(field.ty, Ty::SystemAccount) {
            return Err(ParseError::new(
                field.ident.span(),
                "Cannot use `init` on a `SystemAccount`. \
                    The `SystemAccount` type represents an already-existing account \
                    owned by the system program and cannot be initialized. \
                    If you need to create a new account, use a more specific account type \
                    or `UncheckedAccount` and perform manual initialization instead.",
            ));
        }
    }

    if !init_fields.is_empty() {
        // init needs system program.

        if !fields
            .iter()
            // ensures that a non optional `system_program` is present with non optional `init`
            .any(|f| f.ident() == "system_program" && !(required_init && f.is_optional()))
        {
            return Err(ParseError::new(
                init_fields[0].ident.span(),
                message("init", "system_program", required_init),
            ));
        }

        let kind = &init_fields[0].constraints.init.as_ref().unwrap().kind;
        // init token/a_token/mint needs token program.
        match kind {
            InitKind::Program { .. } | InitKind::Interface { .. } => (),
            InitKind::Token { token_program, .. }
            | InitKind::AssociatedToken { token_program, .. }
            | InitKind::Mint { token_program, .. } => {
                // is the token_program constraint specified?
                let token_program_field = if let Some(token_program_id) = token_program {
                    // if so, is it present in the struct?
                    token_program_id.to_token_stream().to_string()
                } else {
                    // if not, look for the token_program field
                    "token_program".to_string()
                };
                if !fields.iter().any(|f| {
                    f.ident() == &token_program_field && !(required_init && f.is_optional())
                }) {
                    return Err(ParseError::new(
                        init_fields[0].ident.span(),
                        message("init", &token_program_field, required_init),
                    ));
                }
            }
        }

        // a_token needs associated token program.
        if let InitKind::AssociatedToken { .. } = kind {
            if !fields.iter().any(|f| {
                f.ident() == "associated_token_program" && !(required_init && f.is_optional())
            }) {
                return Err(ParseError::new(
                    init_fields[0].ident.span(),
                    message("init", "associated_token_program", required_init),
                ));
            }
        }

        for (pos, field) in init_fields.iter().enumerate() {
            // Get payer for init-ed account
            let associated_payer_name = match field.constraints.init.clone().unwrap().payer {
                // composite payer, check not supported
                Expr::Field(_) => continue,
                // method call, check not supported
                Expr::MethodCall(_) => continue,
                field_name => field_name.to_token_stream().to_string(),
            };

            // Check payer is mutable
            let associated_payer_field = fields.iter().find_map(|f| match f {
                AccountField::Field(field) if *f.ident() == associated_payer_name => Some(field),
                _ => None,
            });
            match associated_payer_field {
                Some(associated_payer_field) => {
                    if !associated_payer_field.constraints.is_mutable() {
                        return Err(ParseError::new(
                            field.ident.span(),
                            "the payer specified for an init constraint must be mutable.",
                        ));
                    } else if associated_payer_field.is_optional && required_init {
                        return Err(ParseError::new(
                            field.ident.span(),
                            "the payer specified for a required init constraint must be required.",
                        ));
                    }
                }
                _ => {
                    return Err(ParseError::new(
                        field.ident.span(),
                        "the payer specified does not exist.",
                    ));
                }
            }
            match &field.constraints.init.as_ref().unwrap().kind {
                // This doesn't catch cases like account.key() or account.key.
                // My guess is that doesn't happen often and we can revisit
                // this if I'm wrong.
                InitKind::Token { mint, .. } | InitKind::AssociatedToken { mint, .. } => {
                    if !fields.iter().any(|f| {
                        f.ident()
                            .to_string()
                            .starts_with(&mint.to_token_stream().to_string())
                    }) {
                        return Err(ParseError::new(
                            field.ident.span(),
                            "the mint constraint has to be an account field for token initializations (not a public key)",
                        ));
                    }
                }

                // Make sure initialized token accounts are always declared after their corresponding mint.
                InitKind::Mint { .. } => {
                    if init_fields.iter().enumerate().any(|(f_pos, f)| {
                        match &f.constraints.init.as_ref().unwrap().kind {
                            InitKind::Token { mint, .. }
                            | InitKind::AssociatedToken { mint, .. } => {
                                field.ident == mint.to_token_stream().to_string() && pos > f_pos
                            }
                            _ => false,
                        }
                    }) {
                        return Err(ParseError::new(
                            field.ident.span(),
                            "because of the init constraint, the mint has to be declared before the corresponding token account",
                        ));
                    }
                }
                _ => (),
            }
        }
    }

    // REALLOC
    let mut required_realloc = false;
    let realloc_fields: Vec<&Field> = fields
        .iter()
        .filter_map(|f| match f {
            AccountField::Field(field) if field.constraints.realloc.is_some() => {
                if !field.is_optional {
                    required_realloc = true
                }
                Some(field)
            }
            _ => None,
        })
        .collect();

    if !realloc_fields.is_empty() {
        // realloc needs system program.
        if !fields
            .iter()
            .any(|f| f.ident() == "system_program" && !(required_realloc && f.is_optional()))
        {
            return Err(ParseError::new(
                realloc_fields[0].ident.span(),
                message("realloc", "system_program", required_realloc),
            ));
        }

        for field in realloc_fields {
            // Get allocator for realloc-ed account
            let associated_payer_name = match field.constraints.realloc.clone().unwrap().payer {
                // composite allocator, check not supported
                Expr::Field(_) => continue,
                // method call, check not supported
                Expr::MethodCall(_) => continue,
                field_name => field_name.to_token_stream().to_string(),
            };

            // Check allocator is mutable
            let associated_payer_field = fields.iter().find_map(|f| match f {
                AccountField::Field(field) if *f.ident() == associated_payer_name => Some(field),
                _ => None,
            });

            match associated_payer_field {
                Some(associated_payer_field) => {
                    if !associated_payer_field.constraints.is_mutable() {
                        return Err(ParseError::new(
                            field.ident.span(),
                            "the realloc::payer specified for an realloc constraint must be mutable.",
                        ));
                    } else if associated_payer_field.is_optional && required_realloc {
                        return Err(ParseError::new(
                            field.ident.span(),
                            "the realloc::payer specified for a required realloc constraint must be required.",
                        ));
                    }
                }
                _ => {
                    return Err(ParseError::new(
                        field.ident.span(),
                        "the realloc::payer specified does not exist.",
                    ));
                }
            }
        }
    }

    Ok(())
}

pub fn parse_account_field(f: &syn::Field) -> ParseResult<AccountField> {
    let ident = f.ident.clone().unwrap();
    let docs = docs::parse(&f.attrs);
    let account_field = match is_field_primitive(f)? {
        true => {
            let (ty, is_optional) = parse_ty(f)?;
            let account_constraints = constraints::parse(f, Some(&ty))?;

            // Validate that wrapper types are not combined with init/zero/realloc
            validate_wrapper_constraint_compatibility(&ty, &account_constraints, f.ty.span())?;

            AccountField::Field(Field {
                ident,
                ty,
                is_optional,
                constraints: account_constraints,
                ty_span: f.ty.span(),
                docs,
            })
        }
        false => {
            let (_, optional, _) = ident_string(f)?;
            if optional {
                return Err(ParseError::new(
                    f.ty.span(),
                    "Cannot have Optional composite accounts",
                ));
            }
            let account_constraints = constraints::parse(f, None)?;
            AccountField::CompositeField(CompositeField {
                ident,
                constraints: account_constraints,
                symbol: ident_string(f)?.0,
                raw_field: f.clone(),
                docs,
            })
        }
    };
    Ok(account_field)
}

fn is_field_primitive(f: &syn::Field) -> ParseResult<bool> {
    let r = matches!(
        ident_string(f)?.0.as_str(),
        "Sysvar"
            | "AccountInfo"
            | "UncheckedAccount"
            | "AccountLoader"
            | "Account"
            | "LazyAccount"
            | "Migration"
            | "Program"
            | "Interface"
            | "InterfaceAccount"
            | "Signer"
            | "SystemAccount"
            | "ProgramData"
            // Wrapper types from account_set module
            | "Mut"
            | "Seeded"
            | "Owned"
            | "Executable"
            | "HasOne"
    );
    Ok(r)
}

fn parse_ty(f: &syn::Field) -> ParseResult<(Ty, bool)> {
    let (ident, optional, path) = ident_string(f)?;
    let ty = match ident.as_str() {
        "Sysvar" => Ty::Sysvar(parse_sysvar(&path)?),
        "AccountInfo" => Ty::AccountInfo,
        "UncheckedAccount" => Ty::UncheckedAccount,
        "AccountLoader" => Ty::AccountLoader(parse_program_account_loader(&path)?),
        "Account" => Ty::Account(parse_account_ty(&path)?),
        "LazyAccount" => Ty::LazyAccount(parse_lazy_account_ty(&path)?),
        "Migration" => Ty::Migration(parse_migration_ty(&path)?),
        "Program" => Ty::Program(parse_program_ty(&path)?),
        "Interface" => Ty::Interface(parse_interface_ty(&path)?),
        "InterfaceAccount" => Ty::InterfaceAccount(parse_interface_account_ty(&path)?),
        "Signer" => Ty::Signer,
        "SystemAccount" => Ty::SystemAccount,
        "ProgramData" => Ty::ProgramData,
        // Wrapper types from account_set module
        "Mut" => Ty::Mut(parse_mut_ty(&path)?),
        "Seeded" => Ty::Seeded(parse_seeded_ty(&path)?),
        "Owned" => Ty::Owned(parse_owned_ty(&path)?),
        "Executable" => Ty::Executable(parse_executable_ty(&path)?),
        "HasOne" => Ty::HasOne(parse_has_one_ty(&path)?),
        _ => return Err(ParseError::new(f.ty.span(), "invalid account type given")),
    };

    Ok((ty, optional))
}

fn option_to_inner_path(path: &Path) -> ParseResult<Path> {
    let segment_0 = path.segments[0].clone();
    match segment_0.arguments {
        syn::PathArguments::AngleBracketed(args) => {
            if args.args.len() != 1 {
                return Err(ParseError::new(
                    args.args.span(),
                    "can only have one argument in option",
                ));
            }
            match &args.args[0] {
                syn::GenericArgument::Type(syn::Type::Path(ty_path)) => Ok(ty_path.path.clone()),
                _ => Err(ParseError::new(
                    args.args[1].span(),
                    "first bracket argument must be a lifetime",
                )),
            }
        }
        _ => Err(ParseError::new(
            segment_0.arguments.span(),
            "expected angle brackets with a lifetime and type",
        )),
    }
}

fn ident_string(f: &syn::Field) -> ParseResult<(String, bool, Path)> {
    let mut path = match &f.ty {
        syn::Type::Path(ty_path) => ty_path.path.clone(),
        _ => return Err(ParseError::new(f.ty.span(), "invalid account type given")),
    };
    let mut optional = false;
    if parser::tts_to_string(&path)
        .replace(' ', "")
        .starts_with("Option<")
    {
        path = option_to_inner_path(&path)?;
        optional = true;
    }
    if parser::tts_to_string(&path)
        .replace(' ', "")
        .starts_with("Box<Account<")
    {
        return Ok(("Account".to_string(), optional, path));
    }
    if parser::tts_to_string(&path)
        .replace(' ', "")
        .starts_with("Box<InterfaceAccount<")
    {
        return Ok(("InterfaceAccount".to_string(), optional, path));
    }
    // TODO: allow segmented paths.
    if path.segments.len() != 1 {
        return Err(ParseError::new(
            f.ty.span(),
            "segmented paths are not currently allowed",
        ));
    }

    let segments = &path.segments[0];
    Ok((segments.ident.to_string(), optional, path))
}

fn parse_program_account_loader(path: &syn::Path) -> ParseResult<AccountLoaderTy> {
    let account_ident = parse_account(path)?;
    Ok(AccountLoaderTy {
        account_type_path: account_ident,
    })
}

fn parse_account_ty(path: &syn::Path) -> ParseResult<AccountTy> {
    let account_type_path = parse_account(path)?;
    let boxed = parser::tts_to_string(path)
        .replace(' ', "")
        .starts_with("Box<Account<");
    Ok(AccountTy {
        account_type_path,
        boxed,
    })
}

fn parse_lazy_account_ty(path: &syn::Path) -> ParseResult<LazyAccountTy> {
    let account_type_path = parse_account(path)?;
    Ok(LazyAccountTy { account_type_path })
}

fn parse_migration_ty(path: &syn::Path) -> ParseResult<MigrationTy> {
    // Migration<'info, From, To>
    let segments = &path.segments[0];
    match &segments.arguments {
        syn::PathArguments::AngleBracketed(args) => {
            // Expected: <'info, From, To> - 3 args
            if args.args.len() != 3 {
                return Err(ParseError::new(
                    args.args.span(),
                    "Migration requires three arguments: lifetime, From type, and To type",
                ));
            }
            // First arg is lifetime, second is From, third is To
            let from_type_path = match &args.args[1] {
                syn::GenericArgument::Type(syn::Type::Path(ty_path)) => ty_path.clone(),
                _ => {
                    return Err(ParseError::new(
                        args.args[1].span(),
                        "From type must be a path",
                    ));
                }
            };
            let to_type_path = match &args.args[2] {
                syn::GenericArgument::Type(syn::Type::Path(ty_path)) => ty_path.clone(),
                _ => {
                    return Err(ParseError::new(
                        args.args[2].span(),
                        "To type must be a path",
                    ));
                }
            };
            Ok(MigrationTy {
                from_type_path,
                to_type_path,
            })
        }
        _ => Err(ParseError::new(
            segments.span(),
            "Migration must have angle bracketed arguments",
        )),
    }
}

fn parse_interface_account_ty(path: &syn::Path) -> ParseResult<InterfaceAccountTy> {
    let account_type_path = parse_account(path)?;
    let boxed = parser::tts_to_string(path)
        .replace(' ', "")
        .starts_with("Box<InterfaceAccount<");
    Ok(InterfaceAccountTy {
        account_type_path,
        boxed,
    })
}

fn parse_program_ty(path: &syn::Path) -> ParseResult<ProgramTy> {
    let account_type_path = parse_program_account(path)?;
    Ok(ProgramTy { account_type_path })
}

fn parse_interface_ty(path: &syn::Path) -> ParseResult<InterfaceTy> {
    let account_type_path = parse_account(path)?;
    Ok(InterfaceTy { account_type_path })
}

// Special parsing function for Program that handles both Program<'info> and Program<'info, T>
fn parse_program_account(path: &syn::Path) -> ParseResult<syn::TypePath> {
    let segments = &path.segments[0];
    match &segments.arguments {
        syn::PathArguments::AngleBracketed(args) => {
            match args.args.len() {
                // Program<'info> - only lifetime, no type parameter
                1 => {
                    // Create a special marker for unit type that gets handled later
                    use syn::{Path, PathSegment, PathArguments};
                    let path_segment = PathSegment {
                        ident: syn::Ident::new("__SolanaProgramUnitType", proc_macro2::Span::call_site()),
                        arguments: PathArguments::None,
                    };

                    Ok(syn::TypePath {
                        qself: None,
                        path: Path {
                            leading_colon: None,
                            segments: std::iter::once(path_segment).collect(),
                        },
                    })
                }
                // Program<'info, T> - lifetime and type
                2 => {
                    match &args.args[1] {
                        syn::GenericArgument::Type(syn::Type::Path(ty_path)) => Ok(ty_path.clone()),
                        _ => Err(ParseError::new(
                            args.args[1].span(),
                            "second bracket argument must be a type",
                        )),
                    }
                }
                _ => Err(ParseError::new(
                    args.args.span(),
                    "Program must have either just a lifetime (Program<'info>) or a lifetime and type (Program<'info, T>)",
                )),
            }
        }
        _ => Err(ParseError::new(
            segments.arguments.span(),
            "expected angle brackets with lifetime or lifetime and type",
        )),
    }
}

// TODO: this whole method is a hack. Do something more idiomatic.
fn parse_account(mut path: &syn::Path) -> ParseResult<syn::TypePath> {
    let path_str = parser::tts_to_string(path).replace(' ', "");
    if path_str.starts_with("Box<Account<") || path_str.starts_with("Box<InterfaceAccount<") {
        let segments = &path.segments[0];
        match &segments.arguments {
            syn::PathArguments::AngleBracketed(args) => {
                // Expected: <'info, MyType>.
                if args.args.len() != 1 {
                    return Err(ParseError::new(
                        args.args.span(),
                        "bracket arguments must be the lifetime and type",
                    ));
                }
                match &args.args[0] {
                    syn::GenericArgument::Type(syn::Type::Path(ty_path)) => {
                        path = &ty_path.path;
                    }
                    _ => {
                        return Err(ParseError::new(
                            args.args[1].span(),
                            "first bracket argument must be a lifetime",
                        ))
                    }
                }
            }
            _ => {
                return Err(ParseError::new(
                    segments.arguments.span(),
                    "expected angle brackets with a lifetime and type",
                ))
            }
        }
    }

    let segments = &path.segments[0];
    match &segments.arguments {
        syn::PathArguments::AngleBracketed(args) => {
            // Expected: <'info, MyType>.
            if args.args.len() != 2 {
                return Err(ParseError::new(
                    args.args.span(),
                    "bracket arguments must be the lifetime and type",
                ));
            }
            match &args.args[1] {
                syn::GenericArgument::Type(syn::Type::Path(ty_path)) => Ok(ty_path.clone()),
                _ => Err(ParseError::new(
                    args.args[1].span(),
                    "first bracket argument must be a lifetime",
                )),
            }
        }
        _ => Err(ParseError::new(
            segments.arguments.span(),
            "expected angle brackets with a lifetime and type",
        )),
    }
}

fn parse_sysvar(path: &syn::Path) -> ParseResult<SysvarTy> {
    let segments = &path.segments[0];
    let account_ident = match &segments.arguments {
        syn::PathArguments::AngleBracketed(args) => {
            // Expected: <'info, MyType>.
            if args.args.len() != 2 {
                return Err(ParseError::new(
                    args.args.span(),
                    "bracket arguments must be the lifetime and type",
                ));
            }
            match &args.args[1] {
                syn::GenericArgument::Type(syn::Type::Path(ty_path)) => {
                    // TODO: allow segmented paths.
                    if ty_path.path.segments.len() != 1 {
                        return Err(ParseError::new(
                            ty_path.path.span(),
                            "segmented paths are not currently allowed",
                        ));
                    }
                    let path_segment = &ty_path.path.segments[0];
                    path_segment.ident.clone()
                }
                _ => {
                    return Err(ParseError::new(
                        args.args[1].span(),
                        "first bracket argument must be a lifetime",
                    ))
                }
            }
        }
        _ => {
            return Err(ParseError::new(
                segments.arguments.span(),
                "expected angle brackets with a lifetime and type",
            ))
        }
    };
    let ty = match account_ident.to_string().as_str() {
        "Clock" => SysvarTy::Clock,
        "Rent" => SysvarTy::Rent,
        "EpochSchedule" => SysvarTy::EpochSchedule,
        "Fees" => SysvarTy::Fees,
        "RecentBlockhashes" => SysvarTy::RecentBlockhashes,
        "SlotHashes" => SysvarTy::SlotHashes,
        "SlotHistory" => SysvarTy::SlotHistory,
        "StakeHistory" => SysvarTy::StakeHistory,
        "Instructions" => SysvarTy::Instructions,
        "Rewards" => SysvarTy::Rewards,
        _ => {
            return Err(ParseError::new(
                account_ident.span(),
                "invalid sysvar provided",
            ))
        }
    };
    Ok(ty)
}

// ============================================================================
// Wrapper type parsing functions
// ============================================================================

/// Parse Mut<T> wrapper type
fn parse_mut_ty(path: &syn::Path) -> ParseResult<MutTy> {
    let inner = parse_wrapper_inner_ty(path, "Mut")?;
    Ok(MutTy {
        inner: Box::new(inner),
    })
}

/// Parse Seeded<T, S> wrapper type
fn parse_seeded_ty(path: &syn::Path) -> ParseResult<SeededTy> {
    let segments = &path.segments[0];
    match &segments.arguments {
        syn::PathArguments::AngleBracketed(args) => {
            // Seeded<T, S> - needs inner type T and seeds type S
            if args.args.len() != 2 {
                return Err(ParseError::new(
                    args.args.span(),
                    "Seeded requires two type arguments: inner account type and seeds type",
                ));
            }

            // First arg is inner type T
            let inner_path = match &args.args[0] {
                syn::GenericArgument::Type(syn::Type::Path(ty_path)) => ty_path.path.clone(),
                _ => {
                    return Err(ParseError::new(
                        args.args[0].span(),
                        "first argument must be an account type",
                    ));
                }
            };
            let inner = parse_ty_from_path(&inner_path)?;

            // Second arg is seeds type S
            let seeds_type_path = match &args.args[1] {
                syn::GenericArgument::Type(syn::Type::Path(ty_path)) => ty_path.clone(),
                _ => {
                    return Err(ParseError::new(
                        args.args[1].span(),
                        "second argument must be a seeds type",
                    ));
                }
            };

            Ok(SeededTy {
                inner: Box::new(inner),
                seeds_type_path,
            })
        }
        _ => Err(ParseError::new(
            segments.arguments.span(),
            "Seeded must have angle bracketed arguments",
        )),
    }
}

/// Parse Owned<T, P> wrapper type
fn parse_owned_ty(path: &syn::Path) -> ParseResult<OwnedTy> {
    let segments = &path.segments[0];
    match &segments.arguments {
        syn::PathArguments::AngleBracketed(args) => {
            // Owned<T, P> - needs inner type T and program type P
            if args.args.len() != 2 {
                return Err(ParseError::new(
                    args.args.span(),
                    "Owned requires two type arguments: inner account type and program type",
                ));
            }

            // First arg is inner type T
            let inner_path = match &args.args[0] {
                syn::GenericArgument::Type(syn::Type::Path(ty_path)) => ty_path.path.clone(),
                _ => {
                    return Err(ParseError::new(
                        args.args[0].span(),
                        "first argument must be an account type",
                    ));
                }
            };
            let inner = parse_ty_from_path(&inner_path)?;

            // Second arg is program type P
            let program_type_path = match &args.args[1] {
                syn::GenericArgument::Type(syn::Type::Path(ty_path)) => ty_path.clone(),
                _ => {
                    return Err(ParseError::new(
                        args.args[1].span(),
                        "second argument must be a program type",
                    ));
                }
            };

            Ok(OwnedTy {
                inner: Box::new(inner),
                program_type_path,
            })
        }
        _ => Err(ParseError::new(
            segments.arguments.span(),
            "Owned must have angle bracketed arguments",
        )),
    }
}

/// Parse Executable<T> wrapper type
fn parse_executable_ty(path: &syn::Path) -> ParseResult<ExecutableTy> {
    let inner = parse_wrapper_inner_ty(path, "Executable")?;
    Ok(ExecutableTy {
        inner: Box::new(inner),
    })
}

/// Parse HasOne<T, Target> wrapper type
fn parse_has_one_ty(path: &syn::Path) -> ParseResult<HasOneTy> {
    let segments = &path.segments[0];
    match &segments.arguments {
        syn::PathArguments::AngleBracketed(args) => {
            // HasOne<T, Target> - needs inner type T and target type
            if args.args.len() != 2 {
                return Err(ParseError::new(
                    args.args.span(),
                    "HasOne requires two type arguments: inner account type and target type",
                ));
            }

            // First arg is inner type T
            let inner_path = match &args.args[0] {
                syn::GenericArgument::Type(syn::Type::Path(ty_path)) => ty_path.path.clone(),
                _ => {
                    return Err(ParseError::new(
                        args.args[0].span(),
                        "first argument must be an account type",
                    ));
                }
            };
            let inner = parse_ty_from_path(&inner_path)?;

            // Second arg is target type
            let target_type_path = match &args.args[1] {
                syn::GenericArgument::Type(syn::Type::Path(ty_path)) => ty_path.clone(),
                _ => {
                    return Err(ParseError::new(
                        args.args[1].span(),
                        "second argument must be a target type",
                    ));
                }
            };

            // Infer target field name from target type name
            // e.g., "AuthorityTarget" -> "authority", "OwnerField" -> "owner_field"
            let target_field_name = infer_target_field_name(&target_type_path);

            Ok(HasOneTy {
                inner: Box::new(inner),
                target_type_path,
                target_field_name,
            })
        }
        _ => Err(ParseError::new(
            segments.arguments.span(),
            "HasOne must have angle bracketed arguments",
        )),
    }
}

/// Infer the target field name from a HasOneTarget type name.
/// Converts PascalCase to snake_case and removes common suffixes like "Target" or "Field".
/// e.g., "AuthorityTarget" -> "authority", "OwnerField" -> "owner"
fn infer_target_field_name(type_path: &syn::TypePath) -> String {
    let type_name = type_path
        .path
        .segments
        .last()
        .map(|s| s.ident.to_string())
        .unwrap_or_default();

    // Remove common suffixes
    let name = type_name
        .strip_suffix("Target")
        .or_else(|| type_name.strip_suffix("Field"))
        .unwrap_or(&type_name);

    // Convert PascalCase to snake_case
    pascal_to_snake_case(name)
}

/// Convert PascalCase to snake_case
fn pascal_to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

/// Parse the inner type from a wrapper like Mut<T> or Seeded<T, S>
fn parse_wrapper_inner_ty(path: &syn::Path, wrapper_name: &str) -> ParseResult<Ty> {
    let segments = &path.segments[0];
    match &segments.arguments {
        syn::PathArguments::AngleBracketed(args) => {
            if args.args.is_empty() {
                return Err(ParseError::new(
                    args.args.span(),
                    format!("{} requires an inner account type", wrapper_name),
                ));
            }

            // Get the inner type path
            let inner_path = match &args.args[0] {
                syn::GenericArgument::Type(syn::Type::Path(ty_path)) => ty_path.path.clone(),
                _ => {
                    return Err(ParseError::new(
                        args.args[0].span(),
                        "argument must be an account type",
                    ));
                }
            };

            parse_ty_from_path(&inner_path)
        }
        _ => Err(ParseError::new(
            segments.arguments.span(),
            format!("{} must have angle bracketed arguments", wrapper_name),
        )),
    }
}

/// Parse a Ty from a path (recursive helper for wrapper types)
fn parse_ty_from_path(path: &syn::Path) -> ParseResult<Ty> {
    // Get the type identifier
    if path.segments.is_empty() {
        return Err(ParseError::new(path.span(), "empty path"));
    }

    let ident = path.segments[0].ident.to_string();

    let ty = match ident.as_str() {
        "Sysvar" => Ty::Sysvar(parse_sysvar(path)?),
        "AccountInfo" => Ty::AccountInfo,
        "UncheckedAccount" => Ty::UncheckedAccount,
        "AccountLoader" => Ty::AccountLoader(parse_program_account_loader(path)?),
        "Account" => Ty::Account(parse_account_ty(path)?),
        "LazyAccount" => Ty::LazyAccount(parse_lazy_account_ty(path)?),
        "Migration" => Ty::Migration(parse_migration_ty(path)?),
        "Program" => Ty::Program(parse_program_ty(path)?),
        "Interface" => Ty::Interface(parse_interface_ty(path)?),
        "InterfaceAccount" => Ty::InterfaceAccount(parse_interface_account_ty(path)?),
        "Signer" => Ty::Signer,
        "SystemAccount" => Ty::SystemAccount,
        "ProgramData" => Ty::ProgramData,
        // Wrapper types can be nested
        "Mut" => Ty::Mut(parse_mut_ty(path)?),
        "Seeded" => Ty::Seeded(parse_seeded_ty(path)?),
        "Owned" => Ty::Owned(parse_owned_ty(path)?),
        "Executable" => Ty::Executable(parse_executable_ty(path)?),
        "HasOne" => Ty::HasOne(parse_has_one_ty(path)?),
        _ => {
            return Err(ParseError::new(
                path.span(),
                format!("invalid account type: {}", ident),
            ));
        }
    };

    Ok(ty)
}

// ============================================================================
// Wrapper type validation
// ============================================================================

/// Check if a type is a wrapper type (Mut, Seeded, Owned, Executable, HasOne)
fn is_wrapper_type(ty: &Ty) -> bool {
    matches!(
        ty,
        Ty::Mut(_) | Ty::Seeded(_) | Ty::Owned(_) | Ty::Executable(_) | Ty::HasOne(_)
    )
}

/// Get the wrapper type name for error messages
fn wrapper_type_name(ty: &Ty) -> &'static str {
    match ty {
        Ty::Mut(_) => "Mut<T>",
        Ty::Seeded(_) => "Seeded<T, S>",
        Ty::Owned(_) => "Owned<T, P>",
        Ty::Executable(_) => "Executable<T>",
        Ty::HasOne(_) => "HasOne<T, Target>",
        _ => "wrapper type",
    }
}

/// Validate that wrapper types are not combined with init/zero/realloc constraints.
/// These constraints are implemented by attribute-specific codegen that expects
/// base account types and init/zero/realloc metadata (space/payer/seeds).
/// Wrappers are not wired into those flows, so combining them is unsupported.
fn validate_wrapper_constraint_compatibility(
    ty: &Ty,
    constraints: &ConstraintGroup,
    span: proc_macro2::Span,
) -> ParseResult<()> {
    // Only validate if this is a wrapper type at the outermost level
    if !is_wrapper_type(ty) {
        return Ok(());
    }

    let wrapper_name = wrapper_type_name(ty);

    // Check for init constraint
    if constraints.init.is_some() {
        return Err(ParseError::new(
            span,
            format!(
                "cannot combine {} wrapper type with #[account(init, ...)]. \
                 Wrapper types do not support account initialization. \
                 Use the base type (e.g., Account<'info, T>) with #[account(init, ...)] instead.",
                wrapper_name
            ),
        ));
    }

    // Check for zeroed constraint
    if constraints.zeroed.is_some() {
        return Err(ParseError::new(
            span,
            format!(
                "cannot combine {} wrapper type with #[account(zero)]. \
                 Wrapper types do not support zero initialization. \
                 Use the base type (e.g., AccountLoader<'info, T>) with #[account(zero)] instead.",
                wrapper_name
            ),
        ));
    }

    // Check for realloc constraint (attribute-based)
    if constraints.realloc.is_some() {
        return Err(ParseError::new(
            span,
            format!(
                "cannot combine {} wrapper type with #[account(realloc = ...)]. \
                 Use the base type with #[account(realloc = ...)] instead.",
                wrapper_name
            ),
        ));
    }

    Ok(())
}
