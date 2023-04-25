use friday_core;
use std::{convert, io};

// See https://kerkour.com/rust-error-handling

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    #[error("IO error: {0}")]
    IO(io::Error),
    #[error("{0}")]
    Core(friday_core::Error),
}

impl convert::From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IO(err)
    }
}

impl convert::From<friday_core::Error> for Error {
    fn from(err: friday_core::Error) -> Self {
        Error::Core(err)
    }
}

// this pattern can also be found in ripgrep and anyhow
pub type Result<T> = std::result::Result<T, Error>;
