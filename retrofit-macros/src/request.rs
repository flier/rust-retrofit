use proc_macro2::TokenStream;
use quote::quote;
use syn::Result;

pub fn head(_attr: TokenStream, item: syn::TraitItemMethod) -> Result<TokenStream> {
    let expanded = quote! { #item };

    Ok(expanded)
}

pub fn get(_attr: TokenStream, item: syn::TraitItemMethod) -> Result<TokenStream> {
    let expanded = quote! { #item };

    Ok(expanded)
}

pub fn post(_attr: TokenStream, item: syn::TraitItemMethod) -> Result<TokenStream> {
    let expanded = quote! { #item };

    Ok(expanded)
}

pub fn put(_attr: TokenStream, item: syn::TraitItemMethod) -> Result<TokenStream> {
    let expanded = quote! { #item };

    Ok(expanded)
}

pub fn patch(_attr: TokenStream, item: syn::TraitItemMethod) -> Result<TokenStream> {
    let expanded = quote! { #item };

    Ok(expanded)
}

pub fn delete(_attr: TokenStream, item: syn::TraitItemMethod) -> Result<TokenStream> {
    let expanded = quote! { #item };

    Ok(expanded)
}
