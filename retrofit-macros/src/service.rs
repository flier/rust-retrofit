use case::CaseExt;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use regex::Regex;
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    Attribute, Error, Expr, ItemTrait, LitStr, Result, Token, TraitItemMethod,
};

lazy_static::lazy_static! {
    static ref RE_FMT_ARG: Regex = Regex::new(r"\{(?P<name>\w+)(:[^\}]+)?\}").unwrap();
}

pub fn service(_attr: TokenStream, item: ItemTrait) -> Result<TokenStream> {
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
            let req = Request::extract(&method)?;
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

            Ok(quote! {
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

    Ok(expanded)
}

#[derive(Clone, Debug)]
enum Request {
    Get { path: LitStr, args: Args },
    Head { path: LitStr, args: Args },
    Patch { path: LitStr, args: Args },
    Post { path: LitStr, args: Args },
    Put { path: LitStr, args: Args },
    Delete { path: LitStr, args: Args },
}

impl Request {
    pub fn extract(method: &TraitItemMethod) -> Result<Self> {
        let args = Args::extract(&method.attrs)?;

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
            method.span(),
            "expected `get`, `post` or other method",
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
            | Request::Delete { args, .. } => &args.0,
        }
    }
}

#[derive(Clone, Debug, Default)]
struct Args(Punctuated<Arg, Token![,]>);

#[derive(Clone, Debug)]
struct Arg {
    pub ident: Ident,
    pub eq_token: Token![=],
    pub expr: Expr,
}

impl Args {
    pub fn extract(attrs: &[Attribute]) -> Result<Self> {
        let args = attrs
            .iter()
            .filter(|attr| attr.path.is_ident("args"))
            .map(|attr| attr.parse_args::<Args>())
            .collect::<Result<Vec<Args>>>()?;

        Ok(Args(
            args.into_iter()
                .flat_map(|args| args.0.into_pairs())
                .collect(),
        ))
    }
}

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Punctuated::parse_terminated(input).map(Args)
    }
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

impl ToTokens for Args {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl ToTokens for Arg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        self.eq_token.to_tokens(tokens);
        self.expr.to_tokens(tokens);
    }
}
