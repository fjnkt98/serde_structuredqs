mod keyvalue;
mod toplevel;

use crate::{error::Result, ser::toplevel::TopLevelSerializer};
use serde::Serialize;

/// Serialize struct into `x-www-form-urlencoded` format string.
/// For fields that have a structure as their value, the field name is concatenated with the key of the structure, resulting in `{parentkey}.{childkey}={value}`.
/// ```
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct ParentStruct {
///     key1: String,
///     key2: ChildStruct,
/// }
///
/// #[derive(Serialize)]
/// struct ChildStruct {
///     key3: i32,
///     key4: String,
/// }
///
/// let param = ParentStruct {
///     key1: String::from("value1"),
///     key2: ChildStruct {
///         key3: 100,
///         key4: String::from("value4")
///     }
/// };
///
/// assert_eq!(serde_structuredqs::to_string(&param).unwrap(), "key1=value1&key2.key3=100&key2.key4=value4");
/// ```
pub fn to_string<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    let mut encoder = form_urlencoded::Serializer::new(String::from(""));
    value.serialize(TopLevelSerializer::new(&mut encoder))?;
    Ok(encoder.finish())
}
