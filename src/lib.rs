//! # serde_structuredqs
//!
//! This crate is a Rust library for serialize/deserialize structured query-string.  
//! This crate was strongly inspired by [serde_urlencoded](https://crates.io/crates/serde_urlencoded) and [serde_qs](https://crates.io/crates/serde_qs).
//!
//! ## Example
//!
//! ### Basic usage
//!
//! Serialize nested struct with period-delimited keys.
//!
//! ```rust
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct Parent {
//!     key1: String,
//!     key2: Option<String>,
//!     child: Child,
//! }
//!
//! #[derive(Serialize)]
//! struct Child {
//!     key3: i32,
//!     key4: Option<i32>,
//! }
//!
//! assert_eq!(
//!     serde_structuredqs::to_string(&Parent {
//!         key1: String::from("foo"),
//!         key2: Some(String::from("ほげ")),
//!         child: Child {
//!             key3: 100,
//!             key4: None,
//!         }
//!     })
//!     .unwrap(),
//!     String::from("key1=foo&key2=%E3%81%BB%E3%81%92&child.key3=100")
//! )
//!  ```
//!
//!  And deserialize query-string with period-delimited keys into nested struct.
//!  ```rust
//!  use serde::Deserialize;
//!
//!  #[derive(Debug, Deserialize, Eq, PartialEq)]
//!  struct Parent {
//!     key1: String,
//!     key2: Option<String>,
//!     child: Child,
//! }
//!
//! #[derive(Debug, Deserialize, Eq, PartialEq)]
//! struct Child {
//!     key3: i32,
//!     key4: Option<i32>,
//! }
//!
//! assert_eq!(
//!     serde_structuredqs::from_str::<Parent>(
//!         "key1=foo&key2=%E3%81%BB%E3%81%92&child.key3=100"
//!     )
//!     .unwrap(),
//!     Parent {
//!         key1: String::from("foo"),
//!         key2: Some(String::from("ほげ")),
//!         child: Child {
//!             key3: 100,
//!             key4: None,
//!         }
//!     }
//!  )
//! ```
//!
//! ### Vec Support
//!
//! The value of a field of type `Vec` is serialized as a comma-separated string.
//!
//! ```rust
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct MyStruct {
//!     key: Vec<i32>,
//! }
//! assert_eq!(
//!     serde_structuredqs::to_string(&MyStruct {
//!         key: vec![100, 200],
//!     }).unwrap(),
//!     String::from("key=100%2C200")
//! )
//! ```
//!
//! Similarly, comma-separated values are deserialized as `Vec`.
//!
//! ```rust
//! use serde::Deserialize;
//!
//! #[derive(Debug, Deserialize, Eq, PartialEq)]
//! struct MyStruct {
//!    key: Vec<String>,
//! }
//!
//! assert_eq!(
//!    serde_structuredqs::from_str::<MyStruct>(
//!        "key=foo%2Cbar%2C%E3%81%BB%E3%81%92%2C%E3%81%B5%E3%81%8C"
//!    )
//!    .unwrap(),
//!     MyStruct {
//!         key: vec![
//!             String::from("foo"),
//!             String::from("bar"),
//!             String::from("ほげ"),
//!             String::from("ふが"),
//!         ]
//!     }
//! )
//! ```

mod de;
mod error;
mod ser;

pub use de::{from_bytes, from_str};
pub use error::{Error, Result};
pub use ser::to_string;
