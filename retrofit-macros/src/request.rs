use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::Result;

pub fn head(_attr: TokenStream, item: syn::TraitItemMethod) -> Result<TokenStream> {
    Ok(item.into_token_stream())
}

pub fn get(_attr: TokenStream, item: syn::TraitItemMethod) -> Result<TokenStream> {
    Ok(item.into_token_stream())
}

pub fn post(_attr: TokenStream, item: syn::TraitItemMethod) -> Result<TokenStream> {
    Ok(item.into_token_stream())
}

pub fn put(_attr: TokenStream, item: syn::TraitItemMethod) -> Result<TokenStream> {
    Ok(item.into_token_stream())
}

pub fn patch(_attr: TokenStream, item: syn::TraitItemMethod) -> Result<TokenStream> {
    Ok(item.into_token_stream())
}

pub fn delete(_attr: TokenStream, item: syn::TraitItemMethod) -> Result<TokenStream> {
    Ok(item.into_token_stream())
}
