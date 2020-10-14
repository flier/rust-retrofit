use std::ops::Deref;

use case::CaseExt;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    Attribute, Expr, ItemTrait, LitStr, Result, Token, TraitItemMethod,
};

pub fn headers(_attr: Headers, item: TraitItemMethod) -> Result<TokenStream> {
    Ok(item.into_token_stream())
}

pub fn default_headers(_attr: Headers, item: ItemTrait) -> Result<TokenStream> {
    Ok(item.into_token_stream())
}

#[derive(Clone, Debug)]
pub struct Headers(Punctuated<Header, Token![,]>);

impl Deref for Headers {
    type Target = Punctuated<Header, Token![,]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Headers {
    pub fn extract<'a>(name: &str, attrs: &'a [Attribute]) -> Result<Self> {
        let id = Ident::new(name, Span::call_site());
        let path = parse_quote! { retrofit::#id };

        attrs
            .iter()
            .filter(|attr| attr.path.is_ident(name) || attr.path == path)
            .map(|attr| {
                attr.parse_args_with(Punctuated::parse_terminated)
                    .map(|args| args.into_pairs())
            })
            .collect::<Result<Vec<_>>>()
            .map(|args| args.into_iter().flatten().collect())
            .map(Headers)
    }
}

#[derive(Clone, Debug)]
pub struct Header {
    pub name: Ident,
    pub eq_token: Token![=],
    pub value: Expr,
}

impl Parse for Headers {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Punctuated::parse_terminated(input).map(Headers)
    }
}

impl Parse for Header {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(Header {
            name: input.parse()?,
            eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl ToTokens for Headers {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let insert_headers = self.0.iter().map(|header| {
            let name = LitStr::new(&header.name.to_string().to_dashed(), Span::call_site());
            let value = match &header.value {
                Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(value),
                    ..
                }) => {
                    quote! {
                        reqwest::header::HeaderValue::from_static(#value)
                    }
                }
                value @ _ => {
                    quote! {
                        reqwest::header::HeaderValue::from_maybe_shared(#value).expect("header value")
                    }
                }
            };

            quote! {
                headers.insert(
                    #name,
                    #value
                );
            }
        });

        let expanded = quote! {
            {
                let mut headers = reqwest::header::HeaderMap::new();
                #(#insert_headers)*
                headers
            }
        };

        expanded.to_tokens(tokens)
    }
}
