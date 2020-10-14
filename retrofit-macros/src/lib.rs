use proc_macro::TokenStream;
use syn::{parse::Error as ParseError, parse_macro_input, DeriveInput};

mod header;
mod request;
mod service;

trait Output {
    fn process(self) -> TokenStream;
}

impl Output for proc_macro2::TokenStream {
    fn process(self) -> TokenStream {
        self.into()
    }
}

impl Output for Result<proc_macro2::TokenStream, ParseError> {
    fn process(self) -> TokenStream {
        match self {
            Ok(ts) => ts.into(),
            Err(e) => e.to_compile_error().into(),
        }
    }
}

#[proc_macro_attribute]
pub fn service(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(service::service(
        syn::parse(attr).expect("args"),
        syn::parse(item).expect("trait"),
    ))
}

#[proc_macro_attribute]
pub fn client(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(service::client(
        syn::parse(attr).expect("args"),
        syn::parse(item).expect("trait"),
    ))
}

#[proc_macro_derive(Body)]
pub fn body(item: TokenStream) -> TokenStream {
    Output::process(service::body(parse_macro_input!(item as DeriveInput)))
}

#[proc_macro_attribute]
pub fn default_headers(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(header::default_headers(
        syn::parse(attr).expect("headers"),
        syn::parse(item).expect("trait"),
    ))
}

#[proc_macro_attribute]
pub fn get(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(request::request(
        syn::parse(attr).expect("path"),
        syn::parse(item).expect("trait fn"),
    ))
}

#[proc_macro_attribute]
pub fn head(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(request::request(
        syn::parse(attr).expect("path"),
        syn::parse(item).expect("trait fn"),
    ))
}

#[proc_macro_attribute]
pub fn post(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(request::request(
        syn::parse(attr).expect("path"),
        syn::parse(item).expect("trait fn"),
    ))
}

#[proc_macro_attribute]
pub fn put(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(request::request(
        syn::parse(attr).expect("path"),
        syn::parse(item).expect("trait fn"),
    ))
}

#[proc_macro_attribute]
pub fn patch(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(request::request(
        syn::parse(attr).expect("path"),
        syn::parse(item).expect("trait fn"),
    ))
}

#[proc_macro_attribute]
pub fn delete(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(request::request(
        syn::parse(attr).expect("path"),
        syn::parse(item).expect("trait fn"),
    ))
}

#[proc_macro_attribute]
pub fn trace(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(request::request(
        syn::parse(attr).expect("path"),
        syn::parse(item).expect("trait fn"),
    ))
}

#[proc_macro_attribute]
pub fn http(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(request::http(
        syn::parse(attr).expect("method"),
        syn::parse(item).expect("trait fn"),
    ))
}

#[proc_macro_attribute]
pub fn options(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(request::request(
        syn::parse(attr).expect("path"),
        syn::parse(item).expect("trait fn"),
    ))
}

#[proc_macro_attribute]
pub fn args(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(request::args(
        syn::parse(attr).expect("args"),
        syn::parse(item).expect("trait fn"),
    ))
}

#[proc_macro_attribute]
pub fn headers(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(header::headers(
        syn::parse(attr).expect("headers"),
        syn::parse(item).expect("trait fn"),
    ))
}

#[proc_macro_attribute]
pub fn request(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(request::args(
        syn::parse(attr).expect("args"),
        syn::parse(item).expect("trait fn"),
    ))
}
