pub use config::*;
pub use error::Error;

mod config;
mod error;

// this pattern can also be found in ripgrep and anyhow
pub type Result<T> = std::result::Result<T, Error>;
