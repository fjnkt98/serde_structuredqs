use crate::{
    de::{
        key::KeyDeserializer,
        level::{Level, LevelDeserializer},
        parser::Parser,
    },
    error::{Error, Result},
};

use serde::de::{self, Error as _};
use serde::forward_to_deserialize_any;

use std::borrow::Cow;
use std::collections::btree_map::{BTreeMap, IntoIter};
use std::iter::Iterator;
use std::str;

/// A deserializer for the querystring format.
///
/// Supported top-level outputs are structs and maps.
pub(crate) struct Deserializer<'a> {
    pub(crate) iter: IntoIter<Cow<'a, str>, Level<'a>>,
    pub(crate) value: Option<Level<'a>>,
}

impl<'a> Deserializer<'a> {
    pub(crate) fn with_map(map: BTreeMap<Cow<'a, str>, Level<'a>>) -> Self {
        Deserializer {
            iter: map.into_iter(),
            value: None,
        }
    }

    /// Returns a new `Deserializer<'a>`.
    pub(crate) fn with_bytes(input: &'a [u8]) -> Result<Self> {
        Parser::new(input).as_deserializer()
    }
}

impl<'de> de::Deserializer<'de> for Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.iter.next().is_none() {
            return visitor.visit_unit();
        }

        Err(Error::custom("primitive"))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    /// Throws an error.
    ///
    /// Sequences are not supported at the top level.
    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::custom("sequence"))
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    /// Throws an error.
    ///
    /// Tuples are not supported at the top level.
    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::custom("tuple"))
    }

    /// Throws an error.
    ///
    /// TupleStructs are not supported at the top level.
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::custom("tuple struct"))
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
        Err(Error::custom("enum"))
    }

    forward_to_deserialize_any! {
        bool
        u8
        u16
        u32
        u64
        i8
        i16
        i32
        i64
        f32
        f64
        char
        str
        string
        unit
        option
        bytes
        byte_buf
        unit_struct
        identifier
        ignored_any
    }
}

impl<'de> de::MapAccess<'de> for Deserializer<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if let Some((key, value)) = self.iter.next() {
            self.value = Some(value);
            seed.deserialize(KeyDeserializer(key)).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        if let Some(v) = self.value.take() {
            seed.deserialize(LevelDeserializer(v))
        } else {
            Err(de::Error::custom(
                "Somehow the map was empty after a non-empty key was returned",
            ))
        }
    }
}
