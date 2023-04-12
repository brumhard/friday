pub use config::*;
pub use error::Error;
pub use manager::*;
pub use repo::*;
pub use section::*;

mod config;
mod error;
pub mod http;
mod manager;
mod repo;
mod section;

// this pattern can also be found in ripgrep and anyhow
pub type Result<T> = std::result::Result<T, Error>;
