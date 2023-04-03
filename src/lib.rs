pub use config::*;
pub use error::Error;

mod config;
mod error;

pub type Result<T> = std::result::Result<T, Error>;
