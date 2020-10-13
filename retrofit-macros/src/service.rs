use case::CaseExt;
use lazy_static::lazy_static;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use regex::Regex;
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token, Attribute, Error, Expr, ItemTrait, LitStr, Result, Token, TraitItemMethod,
};

lazy_static! {
    static ref RE_FMT_ARG: Regex = Regex::new(r"\{(?P<name>\w+)(:[^\}]+)?\}").unwrap();
}

pub fn service(_attr: TokenStream, mut item: ItemTrait) -> Result<TokenStream> {
    ensure_trait_bound(&mut item.supertraits);
    ensure_method_return_result(&mut item.items);

    let vis = &item.vis;
    let trait_name = &item.ident;
    let fn_name = Ident::new(&trait_name.to_string().to_snake(), Span::call_site());
    let client_name = Ident::new(&format!("{}Client", trait_name), Span::call_site());

    let methods = generate_methods(&item.items);

    let default_headers = extract_headers("default_headers", &item.attrs)?;
    let default_headers = generate_header_map(default_headers);

    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
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
                    .default_headers(#default_headers)
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

    Ok(expanded)
}

fn ensure_trait_bound(supertraits: &mut Punctuated<syn::TypeParamBound, token::Add>) {
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

            let headers = match extract_headers("headers", &method.attrs) {
                Ok(headers) => {
                    if headers.is_empty() {
                        None
                    } else {
                        let headers = generate_header_map(headers);

                        Some(quote! {
                            req = req.headers(#headers);
                        })
                    }
                }
                Err(err) => Some(err.to_compile_error()),
            };

            quote! {
                #sig {
                    let url = format!(
                        concat!("{base_url}", #path),
                        base_url = self.base_url,
                        #args
                    );
                    let mut req = self.client.get(&url);
                    #headers
                    let res = req.send()?;
                    res.json()
                }
            }
        })
}

#[derive(Clone, Debug)]
struct Request {
    method: http::Method,
    path: LitStr,
    args: Punctuated<Arg, Token![,]>,
}

impl Request {
    pub fn extract(method: &TraitItemMethod) -> Result<Self> {
        let args = extract_args(&method.attrs)?;

        for attr in &method.attrs {
            if attr.path.is_ident("get") || attr.path == parse_quote! { retrofit::get } {
                return attr.parse_args().map(|path| Request {
                    method: http::Method::GET,
                    path,
                    args,
                });
            } else if attr.path.is_ident("head") || attr.path == parse_quote! { retrofit::head } {
                return attr.parse_args().map(|path| Request {
                    method: http::Method::HEAD,
                    path,
                    args,
                });
            } else if attr.path.is_ident("patch") || attr.path == parse_quote! { retrofit::patch } {
                return attr.parse_args().map(|path| Request {
                    method: http::Method::PATCH,
                    path,
                    args,
                });
            } else if attr.path.is_ident("post") || attr.path == parse_quote! { retrofit::post } {
                return attr.parse_args().map(|path| Request {
                    method: http::Method::POST,
                    path,
                    args,
                });
            } else if attr.path.is_ident("put") || attr.path == parse_quote! { retrofit::put } {
                return attr.parse_args().map(|path| Request {
                    method: http::Method::PUT,
                    path,
                    args,
                });
            } else if attr.path.is_ident("delete") || attr.path == parse_quote! { retrofit::delete }
            {
                return attr.parse_args().map(|path| Request {
                    method: http::Method::DELETE,
                    path,
                    args,
                });
            }
        }

        Err(Error::new(
            method.sig.span(),
            format!(
                "expected `get`, `post` or other request for the `{}` method",
                method.sig.ident
            ),
        ))
    }
}

fn extract_args(attrs: &[Attribute]) -> Result<Punctuated<Arg, Token![,]>> {
    attrs
        .iter()
        .filter(|attr| attr.path.is_ident("args"))
        .map(|attr| {
            attr.parse_args_with(Punctuated::parse_terminated)
                .map(|args| args.into_pairs())
        })
        .collect::<Result<Vec<_>>>()
        .map(|args| args.into_iter().flatten().collect())
}

#[derive(Clone, Debug)]
struct Arg {
    pub ident: Ident,
    pub eq_token: Token![=],
    pub expr: Expr,
}

impl Parse for Arg {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(Arg {
            ident: input.parse()?,
            eq_token: input.parse()?,
            expr: input.parse()?,
        })
    }
}

impl ToTokens for Arg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        self.eq_token.to_tokens(tokens);
        self.expr.to_tokens(tokens);
    }
}

fn extract_headers<'a>(
    name: &str,
    attrs: &'a [Attribute],
) -> Result<Punctuated<Header, token::Comma>> {
    attrs
        .iter()
        .find(|attr| {
            attr.path.is_ident(name) || attr.path == parse_quote! { retrofit::default_headers }
        })
        .map(|attr| attr.parse_args_with(Punctuated::parse_terminated))
        .unwrap_or_else(|| Ok(Punctuated::new()))
}

#[derive(Clone, Debug)]
struct Header {
    pub name: Ident,
    pub eq_token: Token![=],
    pub value: LitStr,
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

fn generate_header_map(headers: impl IntoIterator<Item = Header>) -> TokenStream {
    let insert_headers = headers.into_iter().map(|header| {
        let name = LitStr::new(&header.name.to_string().to_dashed(), Span::call_site());
        let value = header.value;
        quote! {
            headers.insert(
                #name,
                reqwest::header::HeaderValue::from_static(#value)
            );
        }
    });
    quote! {
        {
            let mut headers = reqwest::header::HeaderMap::new();
            #(#insert_headers)*
            headers
        }
    }
}
