use proc_macro::TokenStream;
use syn::parse::Error as ParseError;

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

/// Sets the default headers for every request.
///
/// # Example
///
/// ```
/// #[service(base_url = "https://api.github.com")]
/// #[default_headers(accept = "application/vnd.github.mercy-preview+json")]
/// pub trait Github {
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

/// Sets the query or body for request.
///
/// # Query
///
/// Use `query` to modify the query string of the URL.
///
/// This method appends and does not overwrite. This means that it can be called multiple times
/// and that existing query parameters are not overwritten if the same key is used.
/// The key will simply show up twice in the query string.
/// Calling `&[("foo", "a"), ("foo", "b")]` gives `"foo=a&foo=b"`.
///
/// ## Example
///
/// ```
/// #[service(base_url = "http://httpbin.org")]
/// pub trait HttpBin {
///     #[post("/post")]
///     #[request(query)]
///     fn post<T: Serialize + ?Sized>(&self, query: &T);
/// }
///
/// http_bin().post(&[("lang", "rust")])?;
/// ```
///
/// # JSON
///
/// Use `json` to sets the body to the JSON serialization of the passed value,
/// and also sets the `Content-Type: application/json` header.
///
/// ## Example
///
/// ```
/// #[service(base_url = "http://httpbin.org")]
/// pub trait HttpBin {
///     #[post("/post")]
///     #[request(json = data)]
///     fn post(&self, data: &HashMap<&str, &str>);
/// }
///
/// let mut params = HashMap::new();
/// params.insert("lang", "rust");
/// http_bin().post(&params)?;
/// ```
///
/// # Form
///
/// Use `form` to sets the body to the url encoded serialization of the passed value,
/// and also sets the `Content-Type: application/x-www-form-urlencoded` header.
///
/// ## Example
///
/// ```
/// #[service(base_url = "http://httpbin.org")]
/// pub trait HttpBin {
///     #[post("/post")]
///     #[request(form = data)]
///     fn post(&self, data: &HashMap<&str, &str>);
/// }
///
/// let mut params = HashMap::new();
/// params.insert("lang", "rust");
/// http_bin().post(&params)?;
/// ```
///
/// # Body
///
/// Use `body` to sets the body to the `String`, `File` or bytes.
///
/// ## Example
///
/// Using a `String`:
/// ```
/// #[service(base_url = "http://httpbin.org")]
/// pub trait HttpBin {
///     #[post("/post")]
///     #[request(body = data)]
///     fn post<T: Into<Self::Body>>(&self, data: T);
/// }
///
/// http_bin().post("from a &str!")?;
/// ```
///
/// Using a `File`:
/// ```
/// use std::fs::File;
///
/// #[service(base_url = "http://httpbin.org")]
/// pub trait HttpBin {
///     #[post("/post")]
///     #[request(body = data)]
///     fn post<T: Into<Self::Body>>(&self, data: T);
/// }
///
/// let file = File::open("from_a_file.txt")?;
/// http_bin().post(file)?;
/// ```
///
/// Using arbitrary bytes:
/// ```
/// #[service(base_url = "http://httpbin.org")]
/// pub trait HttpBin {
///     #[post("/post")]
///     #[request(body = data)]
///     fn post<T: Into<Self::Body>>(&self, data: T);
/// }
///
/// let bytes: Vec<u8> = vec![1, 10, 100];
/// http_bin().post(bytes)?;
/// ```
///
/// # Multipart Form
///
/// Use `multipart` to sends a `multipart/form-data` body,
/// a `Form` is built up, adding fields or customized `Part`s.
///
/// ## Example
///
/// ```
/// #[service(base_url = "http://httpbin.org")]
/// pub trait HttpBin {
///     #[post("/post")]
///     #[request(multipart = form)]
///     fn post(&self, form: Self::Form);
/// }
///
/// let form = HttpBin::Form::new()
///     // Adding just a simple text field...
///     .text("username", "seanmonstar")
///     // And a file...
///     .file("photo", "/path/to/photo.png")?;
///
/// http_bin().post(form).unwrap();
/// ```
#[proc_macro_attribute]
pub fn request(attr: TokenStream, item: TokenStream) -> TokenStream {
    Output::process(request::args(
        syn::parse(attr).expect("args"),
        syn::parse(item).expect("trait fn"),
    ))
}
