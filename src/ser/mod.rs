mod keyvalue;
mod toplevel;

use crate::{error::Result, ser::toplevel::TopLevelSerializer};
use serde::Serialize;

pub fn to_string<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    let mut encoder = form_urlencoded::Serializer::new(String::from(""));
    value.serialize(TopLevelSerializer::new(&mut encoder))?;
    Ok(encoder.finish())
}
