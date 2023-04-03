use std::io;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    #[error("IO error: {0}")]
    IO(io::Error),
}

impl std::convert::From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IO(err)
    }
}
