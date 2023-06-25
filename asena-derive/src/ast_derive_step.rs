use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

#[allow(clippy::redundant_clone)]
pub fn expand_ast_derive_step(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident.clone();

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let Data::Struct(data) = input.data else {
        input.ident.span().unwrap().error("A derive `Reporter` should be a struct");
        return TokenStream::new();
    };

    let reporter = data.fields.iter().find_map(|field| {
        let has_reporter = field
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("ast_reporter"));

        if has_reporter {
            let field_name = field.ident.clone();
            Some(quote! {
                impl #impl_generics asena_ast::walker::Reporter for #name #ty_generics #where_clause {
                    fn diagnostic<E: InternalError, T>(&mut self, error: E, at: asena_span::Spanned<T>)
                    where
                        E: 'static,
                    {
                        self.#field_name.diagnostic(error, at);
                    }
                }
            })
        } else {
            None
        }
    });

    TokenStream::from(quote! {
        #reporter
    })
}
