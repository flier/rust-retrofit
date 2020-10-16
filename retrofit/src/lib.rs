pub use retrofit_core::{Call, Service};
pub use retrofit_macros::{args, client, delete, options, patch, post, put, service, trace};

cfg_if::cfg_if! {
    if #[cfg(feature = "reqwest-client")] {
        #[doc(hidden)]
        pub extern crate reqwest;

        pub type Error = reqwest::Error;
        pub type Result<T> = reqwest::Result<T>;
        pub type Method = reqwest::Method;
        pub type HeaderMap = reqwest::header::HeaderMap;
        pub type HeaderValue = reqwest::header::HeaderValue;

        pub mod blocking {
            pub type Client = reqwest::blocking::Client;
            pub type Body = reqwest::blocking::Body;
            pub mod multipart {
                pub type Form = reqwest::blocking::multipart::Form;
            }
        }
    }
}

/// Make a GET request.
///
/// # Example
///
/// ```
/// # use retrofit::{service, get, response};
/// #[service(base_url = "http://httpbin.org")]
/// pub trait HttpBin {
///     #[get("/base64/{value}")]
///     #[response(text())]
///     fn base64(&self, value: &str) -> String;
/// }
///
/// # fn main() -> retrofit::Result<()> {
/// let res = http_bin().base64("SFRUUEJJTiBpcyBhd2Vzb21l")?;
/// assert_eq!(res, "HTTPBIN is awesome");
/// # Ok(()) }
/// ```
pub use retrofit_macros::get;

/// Use a custom HTTP verb for a request.
///
/// # Example
///
/// ```
/// # use retrofit::{service, http};
/// # #[service(base_url = "http://httpbin.org")]
/// # pub trait HttpBin {
/// #[http(custom("/custom/endpoint/"))]
/// fn custom_endpoint(&self);
/// # }
/// ```
pub use retrofit_macros::http;

/// Sets the default headers for every request.
///
/// # Example
///
/// ```
/// # use retrofit::{service, default_headers};
/// #[service(base_url = "https://api.github.com")]
/// #[default_headers(accept = "application/vnd.github.mercy-preview+json")]
/// pub trait Github {
/// }
/// ```
pub use retrofit_macros::default_headers;

/// Adds headers literally supplied in the value.
///
/// **Note**: Headers do not overwrite each other. All headers with the same name will be included in the request.
///
/// # Example
///
/// ```
/// # use retrofit::{service, headers, get};
/// # #[service(base_url = "http://httpbin.org")]
/// # pub trait HttpBin {
/// #[get("/")]
/// #[headers(cache_control = "max-age=640000")]
/// fn root(&self);
/// # }
/// ```
pub use retrofit_macros::headers;

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
/// # use serde::Serialize;
/// # use retrofit::{service, get, request};
/// #[service(base_url = "http://httpbin.org")]
/// pub trait HttpBin {
///     #[get("/get")]
///     #[request(query)]
///     fn get<T: Serialize + ?Sized>(&self, query: &T) -> serde_json::Value;
/// }
///
/// # fn main() -> retrofit::Result<()> {
/// let res = http_bin().get(&[("lang", "rust")])?;
/// assert_eq!(res["args"]["lang"], "rust");
/// # Ok(()) }
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
/// # use std::collections::HashMap;
/// # use retrofit::{service, post, request};
/// #[service(base_url = "http://httpbin.org")]
/// pub trait HttpBin {
///     #[post("/post")]
///     #[request(json = data)]
///     fn post(&self, data: &HashMap<&str, &str>) -> serde_json::Value;
/// }
///
/// # fn main() -> retrofit::Result<()> {
/// let mut params = HashMap::new();
/// params.insert("lang", "rust");
///
/// let res = http_bin().post(&params)?;
/// assert_eq!(res["json"]["lang"], "rust");
/// assert_eq!(res["headers"]["Content-Type"], "application/json");
/// # Ok(()) }
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
/// # use std::collections::HashMap;
/// # use retrofit::{service, post, request};
/// #[service(base_url = "http://httpbin.org")]
/// pub trait HttpBin {
///     #[post("/post")]
///     #[request(form = data)]
///     fn post(&self, data: &HashMap<&str, &str>) -> serde_json::Value;
/// }
///
/// # fn main() -> retrofit::Result<()> {
/// let mut params = HashMap::new();
/// params.insert("lang", "rust");
///
/// let res = http_bin().post(&params)?;
/// assert_eq!(res["form"]["lang"], "rust");
/// assert_eq!(res["headers"]["Content-Type"], "application/x-www-form-urlencoded");
/// # Ok(()) }
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
/// # use retrofit::{service, post, request};
/// #[service(base_url = "http://httpbin.org")]
/// pub trait HttpBin {
///     #[post("/post")]
///     #[request(body = data)]
///     fn post<T: Into<Self::Body>>(&self, data: T) -> serde_json::Value;
/// }
///
/// # fn main() -> retrofit::Result<()> {
/// let res = http_bin().post("from a &str!")?;
/// assert_eq!(res["data"], "from a &str!");
/// # Ok(()) }
/// ```
///
/// Using a `File`:
/// ```,no_run
/// # use std::fs::File;
/// # use retrofit::{service, post, request};
/// #[service(base_url = "http://httpbin.org")]
/// pub trait HttpBin {
///     #[post("/post")]
///     #[request(body = data)]
///     fn post<T: Into<Self::Body>>(&self, data: T) -> serde_json::Value;
/// }
///
/// # fn main() -> anyhow::Result<()> {
/// let file = File::open("from_a_file.txt")?;
/// http_bin().post(file)?;
/// # Ok(()) }
/// ```
///
/// Using arbitrary bytes:
/// ```
/// # use retrofit::{service, post, request};
/// #[service(base_url = "http://httpbin.org")]
/// pub trait HttpBin {
///     #[post("/post")]
///     #[request(body = data)]
///     fn post<T: Into<Self::Body>>(&self, data: T) -> serde_json::Value;
/// }
///
/// # fn main() -> retrofit::Result<()> {
/// let bytes: Vec<u8> = vec![1, 10, 100];
/// let res = http_bin().post(bytes)?;
/// assert_eq!(res["data"].as_str().unwrap(), "\x01\x0a\x64");
/// # Ok(()) }
/// ```
///
/// # Multipart Form
///
/// Use `multipart` to sends a `multipart/form-data` body,
/// a `Form` is built up, adding fields or customized `Part`s.
///
/// ## Example
///
/// ```,no_run
/// # use reqwest::blocking::multipart::Form;
/// # use retrofit::{service, post, request};
/// #[service(base_url = "http://httpbin.org")]
/// pub trait HttpBin {
///     #[post("/post")]
///     #[request(multipart = form)]
///     fn post(&self, form: Self::Form) -> serde_json::Value;
/// }
///
/// # fn main() -> anyhow::Result<()> {
/// let form = Form::new()
///     // Adding just a simple text field...
///     .text("username", "seanmonstar")
///     // And a file...
///     .file("photo", "/path/to/photo.png")?;
///
/// http_bin().post(form)?;
/// # Ok(()) }
/// ```
pub use retrofit_macros::request;

/// Decode response to a submitted request.
///
/// **Notes**: By default, attempts are made to decode the returned content using the JSON encoding.
///
/// # JSON
///
/// Try and deserialize the response body as JSON using serde.
///
/// ## Example
///
/// ```
/// # use retrofit::{service, get, response};
/// # use serde::{Deserialize, Serialize};
/// #[derive(Debug, Clone, Deserialize)]
/// pub struct Ip {
///     origin: String,
/// }
///
/// #[service(base_url = "http://httpbin.org")]
/// pub trait HttpBin {
///     #[get("/ip")]
///     #[response(json())]
///     fn ip(&self) -> Ip;
/// }
///
/// # fn main() -> retrofit::Result<()> {
/// let ip = http_bin().ip()?;
/// assert!(!ip.origin.is_empty());
/// # Ok(()) }
/// ```
///
/// # Text
///
/// Use `text` to get the response text.
///
/// This method decodes the response body with BOM sniffing
/// and with malformed sequences replaced with the REPLACEMENT CHARACTER.
/// Encoding is determinated from the charset parameter of `Content-Type` header, and defaults to utf-8 if not presented.
///
/// ## Example
///
/// ```
/// # use retrofit::{service, get, response};
/// #[service(base_url = "http://httpbin.org")]
/// pub trait HttpBin {
///     #[get("/base64/{value}")]
///     #[response(text())]
///     fn base64(&self, value: &str) -> String;
/// }
///
/// # fn main() -> retrofit::Result<()> {
/// let res = http_bin().base64("SFRUUEJJTiBpcyBhd2Vzb21l")?;
/// assert_eq!(res, "HTTPBIN is awesome");
/// # Ok(()) }
/// ```
///
/// # Text with encoding
///
/// Use `text_with_charset` to get the response text given a specific encoding.
///
/// ## Example
///
/// ```
/// # use retrofit::{service, get, response};
/// #[service(base_url = "http://httpbin.org")]
/// pub trait HttpBin {
///     #[get("/encoding/utf8")]
///     #[response(text_with_charset("utf-8"))]
///     fn utf8_demo(&self) -> String;
/// }
///
/// # fn main() -> retrofit::Result<()> {
/// let res = http_bin().utf8_demo()?;
/// assert!(!res.is_empty());
/// # Ok(()) }
/// ```
///
/// # Binary
///
/// Use `bytes()` to get the full response body as `Bytes`.
///
/// ## Example
///
/// ```
/// # use retrofit::{service, get, headers, response};
/// #[service(base_url = "http://httpbin.org")]
/// pub trait HttpBin {
///     #[get("/bytes/{len}")]
///     #[headers(accept = "accept: application/octet-stream")]
///     #[response(bytes())]
///     fn bytes(&self, len: usize) -> bytes::Bytes;
/// }
///
/// # fn main() -> retrofit::Result<()> {
/// let res = http_bin().bytes(8)?;
/// assert_eq!(res.len(), 8);
/// # Ok(()) }
/// ```
pub use retrofit_macros::response;
