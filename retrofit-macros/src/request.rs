use proc_macro2::TokenStream;
use quote::quote;
use syn::Result;

pub fn get(attr: TokenStream, item: syn::TraitItemMethod) -> Result<TokenStream> {
    let expanded = quote! { #item };

    Ok(expanded.into())
}

pub fn post(attr: TokenStream, item: syn::TraitItemMethod) -> Result<TokenStream> {
    let expanded = quote! { #item };

    Ok(expanded.into())
}

pub fn put(attr: TokenStream, item: syn::TraitItemMethod) -> Result<TokenStream> {
    let expanded = quote! { #item };

    Ok(expanded.into())
}
