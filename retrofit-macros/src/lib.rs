use proc_macro::TokenStream;
use syn::parse::Error as ParseError;

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
        attr.into(),
        syn::parse(item).expect("trait"),
    ))
}

#[proc_macro_attribute]
pub fn get(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(request::get(
        attr.into(),
        syn::parse(item).expect("trait fn"),
    ))
}

#[proc_macro_attribute]
pub fn head(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(request::head(
        attr.into(),
        syn::parse(item).expect("trait fn"),
    ))
}

#[proc_macro_attribute]
pub fn post(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(request::post(
        attr.into(),
        syn::parse(item).expect("trait fn"),
    ))
}

#[proc_macro_attribute]
pub fn put(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(request::put(
        attr.into(),
        syn::parse(item).expect("trait fn"),
    ))
}

#[proc_macro_attribute]
pub fn patch(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(request::patch(
        attr.into(),
        syn::parse(item).expect("trait fn"),
    ))
}

#[proc_macro_attribute]
pub fn delete(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(request::delete(
        attr.into(),
        syn::parse(item).expect("trait fn"),
    ))
}

#[proc_macro_attribute]
pub fn args(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn headers(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn default_headers(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
