use std::ops::Deref;

use case::CaseExt;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_quote, punctuated::Punctuated, ItemTrait, Result, Token};

use crate::{
    header::Headers,
    request::{Arg, Args, Request},
    response,
};

pub fn client(_args: Args, item: ItemTrait) -> Result<TokenStream> {
    Ok(item.into_token_stream())
}

pub fn service(args: Args, mut item: ItemTrait) -> Result<TokenStream> {
    ensure_trait_bound(&mut item.supertraits);

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

    let methods = generate_methods(&mut item.items);

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
                builder: std::cell::RefCell<Option<retrofit::blocking::ClientBuilder>>,
                client: std::cell::RefCell<Option<retrofit::blocking::Client>>,
                init: std::sync::Once,
                base_url: String,
            }

            impl retrofit::Service for #client_name {
                type Error = retrofit::Error;
                type Body = retrofit::blocking::Body;
                type Form = retrofit::blocking::multipart::Form;
            }

            impl #impl_generics #trait_name for #client_name #ty_generics #where_clause {
                #(#methods)*
            }

            static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

            impl #client_name {
                fn with_builder(mut self, builder: retrofit::blocking::ClientBuilder) -> Self {
                    *self.builder.borrow_mut() = Some(builder);
                    self
                }

                fn with_client(mut self, client: retrofit::blocking::Client) -> Self {
                    *self.client.borrow_mut() = Some(client);
                    self
                }

                fn client(&self) -> std::cell::Ref<Option<retrofit::blocking::Client>> {
                    self.init.call_once(|| {
                        if self.client.borrow().is_none() {
                            let mut builder = self.builder.borrow_mut().take().unwrap_or_else(retrofit::blocking::Client::builder)
                                .user_agent(APP_USER_AGENT)
                                #default_headers
                                #(#client_options)*;

                            tracing::trace!(?builder);

                            *self.client.borrow_mut() = Some(builder.build().expect("client"))
                        }
                    });
                    self.client.borrow()
                }
            }

            #client_name {
                builder: std::cell::RefCell::new(None),
                client: std::cell::RefCell::new(None),
                init: std::sync::Once::new(),
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
        supertraits.push(syn::TypeParamBound::Trait(parse_quote! {
            retrofit::Service<
                Error = retrofit::Error,
                Body = retrofit::blocking::Body,
                Form = retrofit::blocking::multipart::Form,
            >
        }));
    }
}

fn generate_methods<'a>(items: &'a mut [syn::TraitItem]) -> impl Iterator<Item = Method> + 'a {
    items
        .iter_mut()
        .flat_map(|item| match item {
            syn::TraitItem::Method(method) if method.default.is_none() => Some(method),
            _ => None,
        })
        .map(|method| {
            match method.sig.output {
                syn::ReturnType::Default => {
                    method.sig.output = parse_quote! { -> Result<(), Self::Error> };
                }
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
            }

            Method(method)
        })
}

struct Method<'a>(&'a syn::TraitItemMethod);

impl<'a> Deref for Method<'a> {
    type Target = syn::TraitItemMethod;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> ToTokens for Method<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let sig = &self.sig;

        let request = {
            let request = Request::extract(self).expect("request");
            let headers = match Headers::extract("headers", &self.attrs) {
                Ok(headers) => {
                    if headers.is_empty() {
                        None
                    } else {
                        Some(quote! { .headers(#headers) })
                    }
                }
                Err(err) => Some(err.to_compile_error()),
            };
            let options = Args::extract("request", &self.attrs)
                .expect("request")
                .into_iter()
                .map(|Arg { ident, expr, .. }| {
                    if let Some(expr) = expr {
                        quote! { .#ident(#expr) }
                    } else {
                        quote! { .#ident(#ident) }
                    }
                });

            quote! {
                #request
                    #headers
                    #(#options)*
            }
        };

        let response = match response::extract(&self.attrs) {
            Ok(Some(decode)) => quote! { res.#decode },
            Ok(None) => quote! { res.json() },
            Err(err) => err.to_compile_error(),
        };

        let expanded = quote! {
            #sig {
                let req = #request;
                tracing::trace!(?req);
                let res = req.send()?;
                tracing::trace!(?res);
                // tracing::trace!(text = %{
                //     let mut buf: Vec<u8> = vec![];
                //     res.copy_to(&mut buf)?;
                //     String::from_utf8(buf).unwrap()
                // });
                #response
            }
        };

        expanded.to_tokens(tokens);
    }
}
