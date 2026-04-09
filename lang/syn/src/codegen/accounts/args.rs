use {crate::AccountsStruct, quote::quote, syn::Expr};

pub fn generate(accs: &AccountsStruct) -> proc_macro2::TokenStream {
    let name = syn::Ident::new(&format!("{}Args", accs.ident), accs.ident.span());

    let fields: Vec<proc_macro2::TokenStream> = match &accs.instruction_api {
        Some(ix_api) => ix_api
            .iter()
            .map(|expr| {
                if let Expr::Type(expr_type) = expr {
                    let name = &expr_type.expr;
                    let ty = &expr_type.ty;
                    quote! {
                        pub #name: #ty
                    }
                } else {
                    panic!("Invalid instruction declaration");
                }
            })
            .collect(),
        None => vec![],
    };

    quote! {
        #[derive(anchor_lang::AnchorSerialize, anchor_lang::AnchorDeserialize, Clone)]
        pub struct #name {
            #(#fields),*
        }
    }
}
