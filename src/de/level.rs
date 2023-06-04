use crate::de::{
    deserializer::Deserializer,
    error::{Error, Result},
    parsablestring::ParsableStringDeserializer,
};

use serde::de;
use serde::forward_to_deserialize_any;

use std::borrow::Cow;
use std::collections::btree_map::{BTreeMap, Entry};
use std::iter::Iterator;
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
                Level::Flat(x) => ParsableStringDeserializer(x).$method(visitor),
                Level::Invalid(e) => Err(de::Error::custom(e)),
                Level::UnInitialized => Err(de::Error::custom(
                    "attempted to deserialize uninitialized value",
                )),
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
                        let _ = o.insert(Level::Invalid(error));
                    }
                    Entry::Vacant(vm) => {
                        // Map is empty, result is None
                        let _ = vm.insert(Level::Flat(value));
                    }
                }
            }
            Level::UnInitialized => {
                let mut map = BTreeMap::default();
                let _ = map.insert(key, Level::Flat(value));
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

impl<'de> de::EnumAccess<'de> for LevelDeserializer<'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        match self.0 {
            Level::Flat(x) => Ok((
                seed.deserialize(ParsableStringDeserializer(x))?,
                LevelDeserializer(Level::Invalid(
                    "this value can only deserialize to a UnitVariant".to_string(),
                )),
            )),
            _ => Err(de::Error::custom(
                "this value can only deserialize to a UnitVariant",
            )),
        }
    }
}

impl<'de> de::VariantAccess<'de> for LevelDeserializer<'de> {
    type Error = Error;
    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }
    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self, visitor)
    }
    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self, visitor)
    }
}

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
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.0 {
            Level::Nested(map) => {
                Deserializer::with_map(map).deserialize_enum(name, variants, visitor)
            }
            Level::Flat(_) => visitor.visit_enum(self),
            x => Err(de::Error::custom(format!(
                "{:?} does not appear to be \
                 an enum",
                x
            ))),
        }
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.0 {
            Level::Nested(_) => self.into_deserializer()?.deserialize_map(visitor),
            Level::Flat(_) => {
                // For a newtype_struct, attempt to deserialize a flat value as a
                // single element sequence.
                visitor.visit_seq(LevelSeq(vec![self.0].into_iter()))
            }
            Level::Invalid(e) => Err(de::Error::custom(e)),
            Level::UnInitialized => Err(de::Error::custom(
                "attempted to deserialize uninitialized value",
            )),
        }
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
        // newtype_struct
        tuple_struct
        struct
        identifier
        tuple
        ignored_any
        seq
        // map
    }
}

pub(crate) struct LevelSeq<'a, I: Iterator<Item = Level<'a>>>(I);

impl<'de, I: Iterator<Item = Level<'de>>> de::SeqAccess<'de> for LevelSeq<'de, I> {
    type Error = Error;
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if let Some(v) = self.0.next() {
            seed.deserialize(LevelDeserializer(v)).map(Some)
        } else {
            Ok(None)
        }
    }
}
