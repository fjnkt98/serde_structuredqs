use crate::error::{Error, Result};

use serde::de::{self, IntoDeserializer};
use serde::forward_to_deserialize_any;

use std::borrow::Cow;
use std::str;

macro_rules! forward_parsable_to_deserialize_any {
    ($($ty:ident => $meth:ident,)*) => {
        $(
            fn $meth<V>(self, visitor: V) -> Result<V::Value> where V: de::Visitor<'de> {
                match self.0.parse::<$ty>() {
                    Ok(val) => val.into_deserializer().$meth(visitor),
                    Err(e) => Err(de::Error::custom(e))
                }
            }
        )*
    }
}

pub(crate) struct KeyDeserializer<'a>(pub Cow<'a, str>);

impl<'de> de::Deserializer<'de> for KeyDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.0.into_deserializer().deserialize_any(visitor)
    }

    forward_to_deserialize_any! {
        map
        struct
        seq
        option
        char
        str
        string
        unit
        bytes
        enum
        byte_buf
        unit_struct
        newtype_struct
        tuple_struct
        identifier
        tuple
        ignored_any
    }

    forward_parsable_to_deserialize_any! {
        bool => deserialize_bool,
        u8 => deserialize_u8,
        u16 => deserialize_u16,
        u32 => deserialize_u32,
        u64 => deserialize_u64,
        i8 => deserialize_i8,
        i16 => deserialize_i16,
        i32 => deserialize_i32,
        i64 => deserialize_i64,
        f32 => deserialize_f32,
        f64 => deserialize_f64,
    }
}
