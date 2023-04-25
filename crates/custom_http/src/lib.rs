#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]

pub use error::*;

pub mod asynchttp;
mod error;
#[deprecated = "use asynchttp instead."]
pub mod http;
mod http_types;
