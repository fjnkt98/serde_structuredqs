use std::{fmt::Display, io, num, string};

use serde::{de, ser};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Custom(String),
    #[error("failed to parse with error: '{0}' at position: {1}")]
    Parse(String, usize),
    #[error("unsupported type for serialization")]
    Unsupported,
    #[error(transparent)]
    FromUtf8(#[from] string::FromUtf8Error),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    ParseInt(#[from] num::ParseIntError),
    #[error("utf8")]
    Utf8(#[from] std::str::Utf8Error),
}

impl Error {
    pub fn top_level(object: &'static str) -> Self {
        Error::Custom(format!("cannot deserialize {} at the top level", object))
    }

    pub fn parse_error<T>(msg: T, position: usize) -> Self
    where
        T: Display,
    {
        Error::Parse(msg.to_string(), position)
    }
}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Custom(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Custom(msg.to_string())
    }
}
