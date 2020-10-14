use case::CaseExt;
use lazy_static::lazy_static;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use regex::Regex;
use syn::{parse_quote, punctuated::Punctuated, DeriveInput, ItemTrait, LitByteStr, Result, Token};

use crate::{
    header::Headers,
    request::{Arg, Args, Request},
};

lazy_static! {
    static ref RE_FMT_ARG: Regex = Regex::new(r"\{(?P<name>\w+)(:[^\}]+)?\}").unwrap();
}

pub fn client(_args: Args, item: ItemTrait) -> Result<TokenStream> {
    Ok(item.into_token_stream())
}

pub fn body(item: DeriveInput) -> Result<TokenStream> {
    let ty = &item.ident;

    Ok(quote! {
        impl From<#ty> for reqwest::blocking::Body {
            fn from(item: #ty) -> Self {
                serde_json::to_string(&item).unwrap().into()
            }
        }
    })
}

pub fn service(args: Args, mut item: ItemTrait) -> Result<TokenStream> {
    ensure_trait_bound(&mut item.supertraits);
    ensure_method_return_result(&mut item.items);

    let client_options =
        Args::extract("client", &item.attrs)?
            .into_iter()
            .map(|Arg { ident, expr, .. }| {
                quote! {
                    .#ident(#expr)
                }
            });
    let service_options = args.into_iter().flat_map(|Arg { ident, expr, .. }| {
        expr.map(|expr| {
            quote! {
                #ident: #expr.into(),
            }
        })
    });

    let vis = &item.vis;
    let trait_name = &item.ident;
    let fn_name = Ident::new(&trait_name.to_string().to_snake(), Span::call_site());
    let client_name = Ident::new(&format!("{}Client", trait_name), Span::call_site());

    let methods = generate_methods(&item.items);

    let default_headers = Headers::extract("default_headers", &item.attrs)?;
    let default_headers = if default_headers.is_empty() {
        None
    } else {
        Some(quote! { .default_headers(#default_headers) })
    };

    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    let impl_fn = quote! {
        #vis fn #fn_name() -> impl #trait_name {
            struct #client_name {
                client: reqwest::blocking::Client,
                base_url: String,
            }

            impl retrofit::Service for #client_name {
                type Error = reqwest::Error;
                type Body = reqwest::blocking::Body;
            }

            impl #impl_generics #trait_name for #client_name #ty_generics #where_clause {
                #(#methods)*
            }

            static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

            let mut builder = reqwest::blocking::Client::builder()
                .user_agent(APP_USER_AGENT)
                #default_headers
                #(#client_options)*;

            tracing::trace!(?builder);

            #client_name {
                client: builder.build().expect("client"),
                #(#service_options)*
            }
        }
    };

    let expanded = quote! {
        #item
        #impl_fn
    };

    Ok(expanded)
}

fn ensure_trait_bound(supertraits: &mut Punctuated<syn::TypeParamBound, Token![+]>) {
    let bounded = supertraits.iter().any(|t| match t {
        syn::TypeParamBound::Trait(syn::TraitBound { path, .. }) => {
            path.is_ident("Service") || *path == parse_quote! { retrofit::Service }
        }
        _ => false,
    });

    if !bounded {
        supertraits.push(syn::TypeParamBound::Trait(
            parse_quote! { retrofit::Service },
        ));
    }
}

fn ensure_method_return_result<'a>(items: &'a mut [syn::TraitItem]) {
    for item in items.iter_mut() {
        match item {
            syn::TraitItem::Method(method) if method.default.is_none() => match method.sig.output {
                syn::ReturnType::Type(_, ref mut ty) => {
                    let return_result = match ty.as_ref() {
                        syn::Type::Path(syn::TypePath { path, .. }) if path.is_ident("Result") => {
                            true
                        }
                        _ => false,
                    };

                    if !return_result {
                        let return_type = ty.as_ref();
                        *ty = Box::new(parse_quote! { Result<#return_type, Self::Error> })
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn generate_methods<'a>(items: &'a [syn::TraitItem]) -> impl Iterator<Item = TokenStream> + 'a {
    items
        .iter()
        .flat_map(|item| match item {
            syn::TraitItem::Method(method) if method.default.is_none() => Some(method),
            _ => None,
        })
        .map(|method| {
            let sig = &method.sig;
            let req = Request::extract(&method).expect("request");
            let http_method = match req.method {
                http::Method::GET
                | http::Method::DELETE
                | http::Method::HEAD
                | http::Method::OPTIONS
                | http::Method::PATCH
                | http::Method::POST
                | http::Method::TRACE
                | http::Method::PUT => {
                    let method = Ident::new(req.method.as_str(), Span::call_site());

                    quote! {
                        reqwest::Method::#method
                    }
                }
                _ => {
                    let method = LitByteStr::new(req.method.as_str().as_bytes(), Span::call_site());

                    quote! {
                        reqwest::Method::from_bytes(#method).expect("method")
                    }
                }
            };
            let path = req.path;
            let args = {
                let fmt = path.value();
                let args = req.args;
                let args = args.iter().cloned().chain(
                    RE_FMT_ARG
                        .captures_iter(&fmt)
                        .flat_map(|cap| {
                            let name = &cap["name"];
                            if args.iter().any(|arg| arg.ident == name) {
                                None
                            } else {
                                Some(Ident::new(name, Span::call_site()))
                            }
                        })
                        .map(|id| {
                            parse_quote! { #id = #id }
                        }),
                );

                quote! { #(#args),* }
            };

            let headers = match Headers::extract("headers", &method.attrs) {
                Ok(headers) => {
                    if headers.is_empty() {
                        None
                    } else {
                        Some(quote! {
                            req = req.headers(#headers);
                        })
                    }
                }
                Err(err) => Some(err.to_compile_error()),
            };

            let options = Args::extract("request", &method.attrs)
                .expect("request")
                .into_iter()
                .map(|Arg { ident, expr, .. }| {
                    if let Some(expr) = expr {
                        quote! {
                            req = req.#ident(#expr);
                        }
                    } else {
                        quote! {
                            req = req.#ident(#ident);
                        }
                    }
                });

            quote! {
                #sig {
                    let url = format!(
                        concat!("{base_url}", #path),
                        base_url = self.base_url,
                        #args
                    );
                    let mut req = self.client.request(#http_method, &url);
                    #headers
                    #(#options)*
                    tracing::trace!(?req);
                    let mut res = req.send()?;
                    tracing::trace!(?res);
                    // tracing::trace!(text = %{
                    //     let mut buf: Vec<u8> = vec![];
                    //     res.copy_to(&mut buf)?;
                    //     String::from_utf8(buf).unwrap()
                    // });
                    res.json()
                }
            }
        })
}
