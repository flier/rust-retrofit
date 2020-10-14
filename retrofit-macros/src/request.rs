use std::result::Result as StdResult;

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token, Attribute, Error, Expr, Ident, LitStr, Result, Token, TraitItemMethod,
};

pub fn request(_attr: LitStr, item: TraitItemMethod) -> Result<TokenStream> {
    Ok(item.into_token_stream())
}

pub fn http(_attr: Http, item: TraitItemMethod) -> Result<TokenStream> {
    Ok(item.into_token_stream())
}

pub fn args(_attr: Args, item: TraitItemMethod) -> Result<TokenStream> {
    Ok(item.into_token_stream())
}

pub struct Http {
    pub method: Ident,
    pub paren_token: token::Paren,
    pub path: LitStr,
}

impl Parse for Http {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;

        Ok(Http {
            method: input.parse()?,
            paren_token: parenthesized!(content in input),
            path: content.parse()?,
        })
    }
}

impl Http {
    pub fn method(&self) -> StdResult<http::Method, http::method::InvalidMethod> {
        http::Method::from_bytes(self.method.to_string().to_uppercase().as_bytes())
    }
}

#[derive(Clone, Debug)]
pub struct Request {
    pub method: http::Method,
    pub path: LitStr,
    pub args: Punctuated<Arg, Token![,]>,
}

impl Request {
    pub fn extract(method: &TraitItemMethod) -> Result<Self> {
        let args = Args::extract("args", &method.attrs)?;

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
            } else if attr.path.is_ident("patch") || attr.path == parse_quote! { retrofit::patch } {
                return attr.parse_args().map(|path| Request {
                    method: http::Method::PATCH,
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
            } else if attr.path.is_ident("trace") || attr.path == parse_quote! { retrofit::trace } {
                return attr.parse_args().map(|path| Request {
                    method: http::Method::TRACE,
                    path,
                    args,
                });
            } else if attr.path.is_ident("options")
                || attr.path == parse_quote! { retrofit::options }
            {
                return attr.parse_args().map(|path| Request {
                    method: http::Method::OPTIONS,
                    path,
                    args,
                });
            } else if attr.path.is_ident("http") || attr.path == parse_quote! { retrofit::http } {
                let req = attr.parse_args::<Http>()?;

                return Ok(Request {
                    method: req
                        .method()
                        .map_err(|err| Error::new(method.sig.span(), err))?,
                    path: req.path,
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

pub struct Args(Punctuated<Arg, Token![,]>);

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Punctuated::parse_terminated(input).map(Args)
    }
}

impl IntoIterator for Args {
    type Item = Arg;
    type IntoIter = syn::punctuated::IntoIter<Arg>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Args {
    pub fn extract(name: &str, attrs: &[Attribute]) -> Result<Punctuated<Arg, Token![,]>> {
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
    }
}

#[derive(Clone, Debug)]
pub struct Arg {
    pub ident: Ident,
    pub eq_token: Option<Token![=]>,
    pub expr: Option<Expr>,
}

impl Parse for Arg {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let ident = input.parse()?;
        let lookahead = input.lookahead1();
        let (eq_token, expr) = if lookahead.peek(Token![=]) {
            (Some(input.parse()?), Some(input.parse()?))
        } else {
            (None, None)
        };

        Ok(Arg {
            ident,
            eq_token,
            expr,
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
