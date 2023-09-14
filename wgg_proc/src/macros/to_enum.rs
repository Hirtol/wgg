use heck::ToSnakeCase;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields};

pub fn enum_to_variant(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let variants = match &ast.data {
        Data::Enum(v) => &v.variants,
        _ => return Err(syn::Error::new(Span::call_site(), "This macro only supports enums.")),
    };

    let enum_name = &ast.ident;

    let variants: Vec<_> = variants
        .iter()
        .filter_map(|variant| match &variant.fields {
            Fields::Named(_) => None,
            Fields::Unnamed(contained) => {
                let variant_name = &variant.ident;
                let fn_name = format_ident!("to_{}", &variant_name.to_string().to_snake_case());

                Some(quote! {
                    #[must_use]
                    #[inline]
                    pub fn #fn_name(self) -> Option<#contained> {
                        match self {
                            #enum_name::#variant_name(contained) => Some(contained),
                            _ => None
                        }
                    }
                })
            }
            Fields::Unit => None,
        })
        .collect();

    Ok(quote! {
        impl #enum_name {
            #(#variants)*
        }
    })
}
