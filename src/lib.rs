#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]

pub use config::*;
pub use error::Error;
pub use manager::*;
pub use repo::*;
pub use section::*;

pub mod asynchttp;
mod config;
mod error;
mod http_types;
mod manager;
mod repo;
mod section;

#[deprecated = "use asynchttp instead."]
pub mod http;

// this pattern can also be found in ripgrep and anyhow
pub type Result<T> = std::result::Result<T, Error>;
