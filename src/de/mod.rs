pub mod parse;

use crate::error::{Error, Result};

use serde::{
    de::{self, Error as _, IntoDeserializer},
    forward_to_deserialize_any,
};

use std::borrow::Cow;
use std::collections::btree_map::{BTreeMap, Entry, IntoIter};

pub fn from_bytes<'de, T: de::Deserialize<'de>>(input: &'de [u8]) -> Result<T> {
    Config::default().deserialize_bytes(input)
}

pub fn from_str<'de, T: de::Deserialize<'de>>(input: &'de str) -> Result<T> {
    from_bytes(input.as_bytes())
}

pub struct Config {
    max_depth: usize,
    strict: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self::new(5, true)
    }
}

impl Config {
    pub fn new(max_depth: usize, strict: bool) -> Self {
        Self { max_depth, strict }
    }

    fn max_depth(&self) -> usize {
        self.max_depth
    }
}

impl Config {
    pub fn deserialize_bytes<'de, T: de::Deserialize<'de>>(&self, input: &'de [u8]) -> Result<T> {
        T::deserialize(QsDeserializer::with_config(self, input)?)
    }

    pub fn deserialize_str<'de, T: de::Deserialize<'de>>(&self, input: &'de str) -> Result<T> {
        self.deserialize_bytes(input.as_bytes())
    }
}

pub struct QsDeserializer<'a> {
    iter: IntoIter<Cow<'a, str>, Level<'a>>,
    value: Option<Level<'a>>,
}

#[derive(Debug)]
enum Level<'a> {
    Nested(BTreeMap<Cow<'a, str>, Level<'a>>),
    OrderedSeq(BTreeMap<usize, Level<'a>>),
    Sequence(Vec<Level<'a>>),
    Flat(Cow<'a, str>),
    Invalid(String),
    Uninitialized,
}

impl<'a> QsDeserializer<'a> {
    fn with_map(map: BTreeMap<Cow<'a, str>, Level<'a>>) -> Self {
        Self {
            iter: map.into_iter(),
            value: None,
        }
    }

    fn with_config(config: &Config, input: &'a [u8]) -> Result<Self> {
        parse::Parser::new(input, config.max_depth(), config.strict).as_deserializer()
    }
}

impl<'de> de::Deserializer<'de> for QsDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.iter.next().is_none() {
            visitor.visit_unit()
        } else {
            Err(Error::custom("top-level not support primitive"))
        }
    }

    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value>
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

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::custom("top-level not support sequence"))
    }

    fn deserialize_newtype_struct<V>(mut self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::custom("top-level not support tuple"))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::custom("top-level not tuple struct"))
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(self)
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

impl<'de> de::MapAccess<'de> for QsDeserializer<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if let Some((key, value)) = self.iter.next() {
            self.value = Some(value);
            let has_bracket = key.contains('[');
            seed.deserialize(ParsableStringDeserializer(key))
                .map(Some)
                .map_err(|e| {
                    if has_bracket {
                        Error::custom("invalid field contains an encoded bracket")
                    } else {
                        e
                    }
                })
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
            Err(Error::custom(
                "somehow the map was empty after a non-empty key was returned",
            ))
        }
    }
}

impl<'de> de::EnumAccess<'de> for QsDeserializer<'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(mut self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        if let Some((key, value)) = self.iter.next() {
            self.value = Some(value);
            Ok((seed.deserialize(ParsableStringDeserializer(key))?, self))
        } else {
            Err(Error::custom("no more values"))
        }
    }
}

impl<'de> de::VariantAccess<'de> for QsDeserializer<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        if let Some(value) = self.value {
            seed.deserialize(LevelDeserializer(value))
        } else {
            Err(Error::custom("no value to deserialize"))
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if let Some(value) = self.value {
            de::Deserializer::deserialize_seq(LevelDeserializer(value), visitor)
        } else {
            Err(Error::custom("no value to deserialize"))
        }
    }

    fn struct_variant<V>(self, _field: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if let Some(value) = self.value {
            de::Deserializer::deserialize_map(LevelDeserializer(value), visitor)
        } else {
            Err(Error::custom("no value to deserialize"))
        }
    }
}

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
            _ => Err(Error::custom(
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

struct LevelSeq<'a, I: Iterator<Item = Level<'a>>>(I);

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

struct LevelDeserializer<'a>(Level<'a>);

macro_rules! deserialize_primitive {
    ($ty:ident, $method:ident, $visit_method:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: de::Visitor<'de>,
        {
            match self.0 {
                Level::Nested(_) => Err(Error::custom(format!(
                    "expected: {:?}, got a map",
                    stringify!($ty)
                ))),
                Level::OrderedSeq(_) => Err(Error::custom(format!(
                    "expected: {:?}, got a ordered sequence",
                    stringify!($ty)
                ))),
                Level::Sequence(_) => Err(Error::custom(format!(
                    "expected: {:?}, got a sequence",
                    stringify!($ty)
                ))),
                Level::Flat(x) => ParsableStringDeserializer(x).$method(visitor),
                Level::Invalid(e) => Err(Error::custom(e)),
                Level::Uninitialized => Err(Error::custom(
                    "attempted to deserialize uninitialized value",
                )),
            }
        }
    };
}

impl<'a> LevelDeserializer<'a> {
    fn into_deserializer(self) -> Result<QsDeserializer<'a>> {
        match self.0 {
            Level::Nested(map) => Ok(QsDeserializer::with_map(map)),
            Level::OrderedSeq(map) => Ok(QsDeserializer::with_map(
                map.into_iter()
                    .map(|(k, v)| (Cow::Owned(k.to_string()), v))
                    .collect(),
            )),
            Level::Invalid(e) => Err(Error::custom(e)),
            l => Err(Error::custom(format!(
                "could not convert {:?} to QsDeserializer<'a>",
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
            Level::OrderedSeq(map) => visitor.visit_seq(LevelSeq(map.into_values())),
            Level::Sequence(seq) => visitor.visit_seq(LevelSeq(seq.into_iter())),
            Level::Flat(x) => match x {
                Cow::Owned(s) => visitor.visit_string(s),
                Cow::Borrowed(s) => visitor.visit_borrowed_str(s),
            },
            Level::Invalid(e) => Err(Error::custom(e)),
            Level::Uninitialized => Err(Error::custom(
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
            _ => Err(Error::custom("expected unit")),
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
                QsDeserializer::with_map(map).deserialize_enum(name, variants, visitor)
            }
            Level::Flat(_) => visitor.visit_enum(self),
            x => Err(Error::custom(format!(
                "{:?} does not appear to be an enum",
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
            Level::OrderedSeq(map) => visitor.visit_seq(LevelSeq(map.into_values())),
            Level::Sequence(seq) => visitor.visit_seq(LevelSeq(seq.into_iter())),
            Level::Flat(_) => visitor.visit_seq(LevelSeq(vec![self.0].into_iter())),
            Level::Invalid(e) => Err(Error::custom(e)),
            Level::Uninitialized => Err(Error::custom(
                "attempted to deserialize uninitialized value",
            )),
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.0 {
            Level::OrderedSeq(_) => self.into_deserializer()?.deserialize_map(visitor),
            _ => self.deserialize_any(visitor),
        }
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

macro_rules! forward_parsable_to_deserialize_any {
    ($($ty:ident => $meth:ident,)*) => {
        $(
            fn $meth<V>(self, visitor: V) -> Result<V::Value>
            where
                V: de::Visitor<'de>
            {
                match self.0.parse::<$ty>() {
                    Ok(val) => val.into_deserializer().$meth(visitor),
                    Err(e) => Err(Error::custom(e))
                }
            }
        )*
    }
}

struct ParsableStringDeserializer<'a>(Cow<'a, str>);

impl<'de> de::Deserializer<'de> for ParsableStringDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.0.into_deserializer().deserialize_any(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(LevelDeserializer(Level::Flat(self.0)))
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
