use crate::{
    de::deserializer::Deserializer,
    error::{Error, Result},
};

use serde::de::{self, Error as _};
use serde::forward_to_deserialize_any;

use std::borrow::Cow;
use std::collections::btree_map::{BTreeMap, Entry};
use std::str;

macro_rules! deserialize_primitive {
    ($ty:ident, $method:ident, $visit_method:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: de::Visitor<'de>,
        {
            match self.0 {
                Level::Nested(_) => Err(de::Error::custom(format!(
                    "Expected: {:?}, got a Map",
                    stringify!($ty)
                ))),
                Level::Invalid(e) => Err(de::Error::custom(e)),
                Level::UnInitialized => Err(de::Error::custom(
                    "attempted to deserialize uninitialized value",
                )),
                Level::Flat(x) => {
                    if let Ok(x) = x.parse::<$ty>() {
                        visitor.$visit_method(x)
                    } else {
                        Err(de::Error::custom(format!(
                            "Expected {:?}, but got {}",
                            stringify!($ty),
                            x
                        )))
                    }
                }
            }
        }
    };
}

#[derive(Debug)]
pub(crate) enum Level<'a> {
    Nested(BTreeMap<Cow<'a, str>, Level<'a>>),
    Flat(Cow<'a, str>),
    Invalid(String),
    UnInitialized,
}

impl<'a> Level<'a> {
    /// If this `Level` value is indeed a map, then attempt to insert
    /// `value` for key `key`.
    /// Returns error if `self` is not a map, or already has an entry for that
    /// key.
    pub fn insert_map_value(&mut self, key: Cow<'a, str>, value: Cow<'a, str>) {
        match *self {
            Level::Nested(ref mut map) => {
                match map.entry(key) {
                    Entry::Occupied(mut o) => {
                        let key = o.key();
                        let error = format!("multiple values for one key: \"{}\"", key);
                        // Throw away old result; map is now invalid anyway.
                        o.insert(Level::Invalid(error));
                    }
                    Entry::Vacant(vm) => {
                        // Map is empty, result is None
                        vm.insert(Level::Flat(value));
                    }
                }
            }
            Level::UnInitialized => {
                let mut map = BTreeMap::default();
                map.insert(key, Level::Flat(value));
                *self = Level::Nested(map);
            }
            _ => {
                *self = Level::Invalid(
                    "attempted to insert map value into non-map structure".to_string(),
                );
            }
        };
    }
}

pub(crate) struct LevelDeserializer<'a>(pub Level<'a>);

impl<'a> LevelDeserializer<'a> {
    fn into_deserializer(self) -> Result<Deserializer<'a>> {
        match self.0 {
            Level::Nested(map) => Ok(Deserializer::with_map(map)),
            Level::Invalid(e) => Err(de::Error::custom(e)),
            l => Err(de::Error::custom(format!(
                "could not convert {:?} to Deserializer<'a>",
                l
            ))),
        }
    }
}

impl<'de> de::Deserializer<'de> for LevelDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.0 {
            Level::Nested(_) => self.into_deserializer()?.deserialize_map(visitor),
            Level::Flat(x) => match x {
                Cow::Owned(s) => visitor.visit_string(s),
                Cow::Borrowed(s) => visitor.visit_borrowed_str(s),
            },
            Level::Invalid(e) => Err(de::Error::custom(e)),
            Level::UnInitialized => Err(de::Error::custom(
                "attempted to deserialize uninitialized value",
            )),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.0 {
            Level::Flat(ref x) if x == "" => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.0 {
            Level::Flat(ref x) if x == "" => visitor.visit_unit(),
            _ => Err(de::Error::custom("expected unit".to_owned())),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::custom("unsupported enum"))
    }

    /// given the hint that this is a map, will first
    /// attempt to deserialize ordered sequences into a map
    /// otherwise, follows the any code path
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    deserialize_primitive!(bool, deserialize_bool, visit_bool);
    deserialize_primitive!(i8, deserialize_i8, visit_i8);
    deserialize_primitive!(i16, deserialize_i16, visit_i16);
    deserialize_primitive!(i32, deserialize_i32, visit_i32);
    deserialize_primitive!(i64, deserialize_i64, visit_i64);
    deserialize_primitive!(u8, deserialize_u8, visit_u8);
    deserialize_primitive!(u16, deserialize_u16, visit_u16);
    deserialize_primitive!(u32, deserialize_u32, visit_u32);
    deserialize_primitive!(u64, deserialize_u64, visit_u64);
    deserialize_primitive!(f32, deserialize_f32, visit_f32);
    deserialize_primitive!(f64, deserialize_f64, visit_f64);

    forward_to_deserialize_any! {
        char
        str
        string
        bytes
        byte_buf
        unit_struct
        newtype_struct
        tuple_struct
        struct
        identifier
        tuple
        ignored_any
        seq
        // map
    }
}
