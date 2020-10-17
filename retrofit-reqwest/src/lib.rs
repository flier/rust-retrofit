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
