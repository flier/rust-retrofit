use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{ItemTrait, LitStr, Result, TraitItemMethod};

use crate::service::{Args, Headers};

pub fn request(_attr: LitStr, item: TraitItemMethod) -> Result<TokenStream> {
    Ok(item.into_token_stream())
}

pub fn args(_attr: Args, item: TraitItemMethod) -> Result<TokenStream> {
    Ok(item.into_token_stream())
}

pub fn headers(_attr: Headers, item: TraitItemMethod) -> Result<TokenStream> {
    Ok(item.into_token_stream())
}

pub fn default_headers(_attr: Headers, item: ItemTrait) -> Result<TokenStream> {
    Ok(item.into_token_stream())
}
