use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    FailedToGetResource(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::FailedToGetResource(item) => write!(f, "Could not get resource: {}", item),
        }
    }
}

impl StdError for Error {}
