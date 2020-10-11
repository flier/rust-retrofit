use std::collections::HashMap;

use case::CaseExt;
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;
use regex::Regex;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Error, ItemTrait, Result, TraitItemMethod,
};

lazy_static::lazy_static! {
    static ref RE_FMT_ARG: Regex = Regex::new(r"\{(?P<name>\w+)(:[^\}]+)?\}").unwrap();
}

pub fn service(attr: TokenStream, item: ItemTrait) -> Result<TokenStream> {
    let vis = &item.vis;
    let trait_name = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let fn_name = Ident::new(&trait_name.to_string().to_snake(), Span::call_site());
    let client_name = Ident::new(&format!("{}Client", trait_name), Span::call_site());

    let methods = item
        .items
        .iter()
        .flat_map(|item| match item {
            syn::TraitItem::Method(method) if method.default.is_none() => Some(method),
            _ => None,
        })
        .map(|method| {
            let sig = &method.sig;
            let args = sig
                .inputs
                .iter()
                .flat_map(|arg| match arg {
                    syn::FnArg::Typed(syn::PatType { pat, ty, .. }) => {
                        match (pat.as_ref(), ty.as_ref()) {
                            (
                                syn::Pat::Ident(syn::PatIdent { ident, .. }),
                                syn::Type::Path(syn::TypePath { path, .. }),
                            ) => Some((ident.to_string(), path)),
                            _ => None,
                        }
                    }
                    _ => None,
                })
                .collect::<HashMap<_, _>>();
            let req = extract_request(&method)?;
            let path = req.path();
            let path_str = path.to_string();
            let fmt_args = RE_FMT_ARG.captures_iter(&path_str).map(|cap| {
                let name = &cap["name"];
                let id = Ident::new(name, Span::call_site());
                match args.get(name) {
                    Some(path) if path.segments.first().unwrap().ident == "Option" => {
                        quote! { #id = #id.map(|v| v.to_string()).unwrap_or_default() }
                    }
                    _ => {
                        quote! { #id = #id }
                    }
                }
            });

            Ok(quote! {
                #sig {
                    let url = format!(
                        concat!("{base_url}", #path),
                        base_url = self.base_url,
                        #(#fmt_args),*
                    );
                    let mut req = self.client.get(&url);
                    let res = req.send()?;
                    res.json()
                }
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let impl_fn = quote! {
        #vis fn #fn_name(base_url: &str) -> impl #trait_name {
            struct #client_name {
                client: reqwest::blocking::Client,
                base_url: String,
            }

            impl retrofit::Service for #client_name {
                type Error = reqwest::Error;
            }

            impl #impl_generics #trait_name for #client_name #ty_generics #where_clause {
                #(#methods)*
            }

            static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

            #client_name {
                client: reqwest::blocking::Client::builder()
                    .user_agent(APP_USER_AGENT)
                    .default_headers({
                        let mut headers = reqwest::header::HeaderMap::new();
                        headers.insert(
                            reqwest::header::ACCEPT,
                            reqwest::header::HeaderValue::from_static("application/vnd.github.v3+json"),
                        );
                        headers
                    })
                    .build()
                    .expect("client"),
                base_url: base_url.to_string(),
            }
        }
    };

    let expanded = quote! {
        #item
        #impl_fn
    };

    Ok(expanded.into())
}

fn extract_request(method: &TraitItemMethod) -> Result<Request> {
    for attr in &method.attrs {
        if attr.path.is_ident("get") {
            return attr.parse_args().map(Request::Get);
        } else if attr.path.is_ident("head") {
            return attr.parse_args().map(Request::Head);
        } else if attr.path.is_ident("patch") {
            return attr.parse_args().map(Request::Patch);
        } else if attr.path.is_ident("post") {
            return attr.parse_args().map(Request::Post);
        } else if attr.path.is_ident("put") {
            return attr.parse_args().map(Request::Put);
        } else if attr.path.is_ident("delete") {
            return attr.parse_args().map(Request::Delete);
        }
    }

    Err(Error::new(
        method.span(),
        "expected `get`, `post` or other method",
    ))
}

enum Request {
    Get(Get),
    Head(Head),
    Patch(Patch),
    Post(Post),
    Put(Put),
    Delete(Delete),
}

struct Get {
    pub path: Literal,
}
struct Head {
    pub path: Literal,
}
struct Patch {
    pub path: Literal,
}
struct Post {
    pub path: Literal,
}
struct Put {
    pub path: Literal,
}
struct Delete {
    pub path: Literal,
}

impl Request {
    pub fn path(&self) -> &Literal {
        match self {
            Request::Get(Get { path, .. })
            | Request::Head(Head { path, .. })
            | Request::Patch(Patch { path, .. })
            | Request::Post(Post { path, .. })
            | Request::Put(Put { path, .. })
            | Request::Delete(Delete { path, .. }) => path,
        }
    }
}

impl Parse for Get {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(Self {
            path: input.parse()?,
        })
    }
}

impl Parse for Head {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(Self {
            path: input.parse()?,
        })
    }
}

impl Parse for Patch {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(Self {
            path: input.parse()?,
        })
    }
}

impl Parse for Post {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(Self {
            path: input.parse()?,
        })
    }
}

impl Parse for Put {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(Self {
            path: input.parse()?,
        })
    }
}

impl Parse for Delete {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(Self {
            path: input.parse()?,
        })
    }
}
