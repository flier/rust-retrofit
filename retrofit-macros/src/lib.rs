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

/// Use this derive macros on a structure when you want to directly control the request body of a POST/PUT request.
///
/// # Example
///
/// ```
/// #[derive(Clone, Debug, Default, Serialize, Body)]
/// pub struct UpdateRepo {
///     /// The name of the repository.
///     pub name: Option<String>,
///     /// A short description of the repository.
///     pub description: Option<String>,
///     /// A URL with more information about the repository.
///     pub homepage: Option<String>,
///     /// Either `true` to make the repository private or `false` to make it public.
///     pub private: Option<bool>,
/// }
///
/// #[service(base_url = "https://api.github.com")]
/// #[default_headers(accept = "application/vnd.github.v3+json")]
/// pub trait GithubService {
///     /// Update a repository
///     #[patch("/repos/{owner}/{repo}")]
///     #[request(body)]
///     fn update_repo(&self, owner: &str, repo: &str, body: UpdateRepo) -> Repo;
/// }
/// ```
#[proc_macro_derive(Body)]
pub fn body(item: TokenStream) -> TokenStream {
    Output::process(service::body(parse_macro_input!(item as DeriveInput)))
}

/// Sets the default headers for every request.
///
/// # Example
///
/// ```
/// #[service(base_url = "https://api.github.com")]
/// #[default_headers(accept = "application/vnd.github.mercy-preview+json")]
/// pub trait GithubService {
/// }
/// ```
#[proc_macro_attribute]
pub fn default_headers(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(header::default_headers(
        syn::parse(attr).expect("headers"),
        syn::parse(item).expect("trait"),
    ))
}

/// Make a GET request.
#[proc_macro_attribute]
pub fn get(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(request::request(
        syn::parse(attr).expect("path"),
        syn::parse(item).expect("trait fn"),
    ))
}

/// Make a HEAD request.
#[proc_macro_attribute]
pub fn head(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(request::request(
        syn::parse(attr).expect("path"),
        syn::parse(item).expect("trait fn"),
    ))
}

/// Make a POST request.
#[proc_macro_attribute]
pub fn post(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(request::request(
        syn::parse(attr).expect("path"),
        syn::parse(item).expect("trait fn"),
    ))
}

/// Make a PUT request.
#[proc_macro_attribute]
pub fn put(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(request::request(
        syn::parse(attr).expect("path"),
        syn::parse(item).expect("trait fn"),
    ))
}

/// Make a PATCH request.
#[proc_macro_attribute]
pub fn patch(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(request::request(
        syn::parse(attr).expect("path"),
        syn::parse(item).expect("trait fn"),
    ))
}

/// Make a DELETE request.
#[proc_macro_attribute]
pub fn delete(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(request::request(
        syn::parse(attr).expect("path"),
        syn::parse(item).expect("trait fn"),
    ))
}

/// Make a TRACE request.
#[proc_macro_attribute]
pub fn trace(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(request::request(
        syn::parse(attr).expect("path"),
        syn::parse(item).expect("trait fn"),
    ))
}

/// Make a OPTIONS request.
#[proc_macro_attribute]
pub fn options(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(request::request(
        syn::parse(attr).expect("path"),
        syn::parse(item).expect("trait fn"),
    ))
}

/// Use a custom HTTP verb for a request.
///
/// # Example
///
/// ```
/// #[http(custom("/custom/endpoint/"))]
/// fn custom_endpoint(&self);
/// ```
#[proc_macro_attribute]
pub fn http(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(request::http(
        syn::parse(attr).expect("method"),
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

/// Adds headers literally supplied in the value.
///
/// **Note**: Headers do not overwrite each other. All headers with the same name will be included in the request.
///
/// # Example
///
/// ```
/// #[headers(cache_control = "max-age=640000")]
/// #[get("/")]
/// fn root()
/// ```
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
