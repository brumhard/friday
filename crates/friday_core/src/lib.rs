#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]

mod error;
mod manager;
mod repo;
mod section;

pub use error::Error;
pub use manager::*;
pub use repo::*;
pub use section::*;
