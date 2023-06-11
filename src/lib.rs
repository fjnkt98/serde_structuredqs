//! # serde_structuredqs
//!
//! This crate is a Rust library for serialize/deserialize structured query-string.  
//! This crate was strongly inspired by [serde_urlencoded](https://crates.io/crates/serde_urlencoded) and [serde_qs](https://crates.io/crates/serde_qs).
//!
mod de;
mod error;
mod ser;

pub use de::{from_bytes, from_str};
pub use error::{Error, Result};
pub use ser::to_string;
