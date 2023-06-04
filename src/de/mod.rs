pub mod deserializer;
pub mod error;
pub mod level;
pub mod parsablestring;
pub mod parser;
pub mod utility;

use crate::de::{deserializer::Deserializer, error::Result};
use serde::de;

pub fn from_bytes<'de, T: de::Deserialize<'de>>(input: &'de [u8]) -> Result<T> {
    T::deserialize(Deserializer::with_bytes(input)?)
}

pub fn from_str<'de, T: de::Deserialize<'de>>(input: &'de str) -> Result<T> {
    from_bytes(input.as_bytes())
}