use proc_macro::TokenStream;

mod macros;

/// Derive individual functions to directly convert an enum to the associated variant.
///
/// # Example
///
/// ```rust
/// #[derive(wgg_proc::IntoEnumVariant)]
/// pub enum Contents {
///     Stuff(String),
///     Things(i64),
/// }
///
/// let content = Contents::Things(5);
/// assert_eq!(Some(5), content.to_things())
/// ```
#[proc_macro_derive(IntoEnumVariant)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let toks = macros::to_enum::enum_to_variant(&ast).unwrap_or_else(|err| err.to_compile_error());

    toks.into()
}
