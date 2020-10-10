use proc_macro2::TokenStream;
use quote::quote;
use syn::Result;

pub fn service(attr: TokenStream, item: syn::ItemTrait) -> Result<TokenStream> {
    let expanded = quote! { #item };

    Ok(expanded.into())
}
