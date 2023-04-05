pub use config::*;
pub use error::Error;
pub use manager::*;

mod config;
mod error;
mod manager;

// this pattern can also be found in ripgrep and anyhow
pub type Result<T> = std::result::Result<T, Error>;
