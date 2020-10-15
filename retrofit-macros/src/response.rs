use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{parse_quote, Attribute, Expr, Result, TraitItemMethod};

pub fn response(_attr: Expr, item: TraitItemMethod) -> Result<TokenStream> {
    Ok(item.into_token_stream())
}

pub fn extract(attrs: &[Attribute]) -> Result<Option<Expr>> {
    let path = parse_quote! { retrofit::response };
    attrs
        .iter()
        .find(|attr| attr.path.is_ident("response") || attr.path == path)
        .map(|attr| attr.parse_args())
        .transpose()
}
