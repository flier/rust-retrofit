use case::CaseExt;
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

lazy_static::lazy_static! {
    static ref RE_FMT_ARG: Regex = Regex::new(r"\{(?P<name>\w+)(:[^\}]+)?\}").unwrap();
}

pub fn service(_attr: TokenStream, mut item: ItemTrait) -> Result<TokenStream> {
    ensure_trait_bound(&mut item.supertraits);

    let vis = &item.vis;
    let trait_name = &item.ident;
    let fn_name = Ident::new(&trait_name.to_string().to_snake(), Span::call_site());
    let client_name = Ident::new(&format!("{}Client", trait_name), Span::call_site());

    let methods = impl_methods(item.items.iter());

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

fn impl_methods<'a>(
    items: impl Iterator<Item = &'a syn::TraitItem> + 'a,
) -> impl Iterator<Item = TokenStream> + 'a {
    items
        .flat_map(|item| match item {
            syn::TraitItem::Method(method) if method.default.is_none() => Some(method),
            _ => None,
        })
        .map(|method| {
            let sig = &method.sig;
            let req = Request::extract(&method).expect("request");
            let path = req.path();
            let args = {
                let args = req.args();
                let fmt = path.value();
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

            quote! {
                #sig {
                    let url = format!(
                        concat!("{base_url}", #path),
                        base_url = self.base_url,
                        #args
                    );
                    let mut req = self.client.get(&url);
                    let res = req.send()?;
                    res.json()
                }
            }
        })
}

#[derive(Clone, Debug)]
enum Request {
    Get {
        path: LitStr,
        args: Punctuated<Arg, Token![,]>,
    },
    Head {
        path: LitStr,
        args: Punctuated<Arg, Token![,]>,
    },
    Patch {
        path: LitStr,
        args: Punctuated<Arg, Token![,]>,
    },
    Post {
        path: LitStr,
        args: Punctuated<Arg, Token![,]>,
    },
    Put {
        path: LitStr,
        args: Punctuated<Arg, Token![,]>,
    },
    Delete {
        path: LitStr,
        args: Punctuated<Arg, Token![,]>,
    },
}

impl Request {
    pub fn extract(method: &TraitItemMethod) -> Result<Self> {
        let args = extract_args(&method.attrs)?;

        for attr in &method.attrs {
            if attr.path.is_ident("get") {
                return attr.parse_args().map(|path| Request::Get { path, args });
            } else if attr.path.is_ident("head") {
                return attr.parse_args().map(|path| Request::Head { path, args });
            } else if attr.path.is_ident("patch") {
                return attr.parse_args().map(|path| Request::Patch { path, args });
            } else if attr.path.is_ident("post") {
                return attr.parse_args().map(|path| Request::Post { path, args });
            } else if attr.path.is_ident("put") {
                return attr.parse_args().map(|path| Request::Put { path, args });
            } else if attr.path.is_ident("delete") {
                return attr.parse_args().map(|path| Request::Delete { path, args });
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

    pub fn path(&self) -> &LitStr {
        match self {
            Request::Get { path, .. }
            | Request::Head { path, .. }
            | Request::Patch { path, .. }
            | Request::Post { path, .. }
            | Request::Put { path, .. }
            | Request::Delete { path, .. } => &path,
        }
    }

    pub fn args(&self) -> &Punctuated<Arg, Token![,]> {
        match self {
            Request::Get { args, .. }
            | Request::Head { args, .. }
            | Request::Patch { args, .. }
            | Request::Post { args, .. }
            | Request::Put { args, .. }
            | Request::Delete { args, .. } => &args,
        }
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
