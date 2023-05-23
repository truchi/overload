#![allow(unused)]

mod overload;

#[proc_macro_attribute]
pub fn overload(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    overload::overload(syn::parse_macro_input!(item))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
