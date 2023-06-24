use crate::{
    error::{Error, Result},
    ser::keyvalue::KeyValueSerializer,
};
use form_urlencoded::Target;
use serde::{de::Error as _, ser, Serialize};
use std::borrow::Cow;

use super::seq::SeqSerializer;

/// `TopLevelSerializer` takes struct or map and serialize it.
pub struct TopLevelSerializer<'input, 'output, T>
where
    T: Target,
{
    encoder: &'output mut form_urlencoded::Serializer<'input, T>,
    state: State,
}

enum State {
    Init,
    WaitingForKey,
    WaitingForChildKey(Cow<'static, str>),
    WaitingForValue(Cow<'static, str>),
}

impl<'input, 'output, T> TopLevelSerializer<'input, 'output, T>
where
    T: Target,
{
    pub fn new(encoder: &'output mut form_urlencoded::Serializer<'input, T>) -> Self {
        Self {
            encoder,
            state: State::Init,
        }
    }
}

macro_rules! serialize_primitive {
    ($ty:ty, $method:ident) => {
        fn $method(self, value: $ty) -> Result<Self::Ok> {
            match self.state {
                State::Init => Err(Error::custom("top-level serializer supports only struct")),
                State::WaitingForKey => Err(Error::custom("key not found")),
                State::WaitingForChildKey(_) => Err(Error::custom("child key not found")),
                State::WaitingForValue(key) => {
                    let serializer = KeyValueSerializer::new(self.encoder, key);
                    value.serialize(serializer)
                }
            }
        }
    };
}

impl<'input, 'output, T> ser::Serializer for TopLevelSerializer<'input, 'output, T>
where
    T: 'output + Target,
{
    type Ok = &'output mut form_urlencoded::Serializer<'input, T>;
    type Error = Error;

    // type SerializeSeq = ser::Impossible<Self::Ok, Error>;
    type SerializeSeq = SeqSerializer<'input, 'output, T>;

    type SerializeTuple = ser::Impossible<Self::Ok, Error>;
    type SerializeTupleStruct = ser::Impossible<Self::Ok, Error>;
    type SerializeTupleVariant = ser::Impossible<Self::Ok, Error>;
    // TODO: Adapt this to handle map serialization.
    type SerializeMap = ser::Impossible<Self::Ok, Error>;
    type SerializeStruct = Self;
    type SerializeStructVariant = ser::Impossible<Self::Ok, Error>;

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        match self.state {
            State::Init => {
                // Serialize the top-level struct
                Ok(Self {
                    encoder: self.encoder,
                    state: State::WaitingForKey,
                })
            }
            State::WaitingForChildKey(_) => Err(Error::custom("unexpected state")),
            State::WaitingForKey => Err(Error::custom("the key has not yet provided")),
            State::WaitingForValue(key) => {
                // Serialize the field that has a struct as a value
                Ok(Self {
                    encoder: self.encoder,
                    state: State::WaitingForChildKey(key),
                })
            }
        }
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(self.encoder)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        Ok(self.encoder)
    }
    fn serialize_newtype_struct<U>(self, _name: &'static str, value: &U) -> Result<Self::Ok>
    where
        U: ?Sized + Serialize,
    {
        value.serialize(self)
    }
    fn serialize_none(self) -> Result<Self::Ok> {
        Ok(self.encoder)
    }

    fn serialize_some<U>(self, value: &U) -> Result<Self::Ok>
    where
        U: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    serialize_primitive!(bool, serialize_bool);
    serialize_primitive!(i8, serialize_i8);
    serialize_primitive!(i16, serialize_i16);
    serialize_primitive!(i32, serialize_i32);
    serialize_primitive!(i64, serialize_i64);
    serialize_primitive!(u8, serialize_u8);
    serialize_primitive!(u16, serialize_u16);
    serialize_primitive!(u32, serialize_u32);
    serialize_primitive!(u64, serialize_u64);
    serialize_primitive!(f32, serialize_f32);
    serialize_primitive!(f64, serialize_f64);
    serialize_primitive!(char, serialize_char);
    serialize_primitive!(&str, serialize_str);

    fn serialize_bytes(self, _value: &[u8]) -> Result<Self::Ok> {
        Err(Error::custom("top-level serializer supports only struct"))
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        Err(Error::custom("top-level serializer supports only struct"))
    }
    fn serialize_newtype_variant<U>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &U,
    ) -> Result<Self::Ok>
    where
        U: ?Sized + Serialize,
    {
        Err(Error::custom("top-level serializer supports only struct"))
    }
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        match self.state {
            State::Init => Err(Error::custom("top-level serializer supports only struct")),
            State::WaitingForChildKey(_) => Err(Error::custom("unexpected state")),
            State::WaitingForKey => Err(Error::custom("the key has not yet provided")),
            State::WaitingForValue(key) => Ok(SeqSerializer::new(self.encoder, key, len)),
        }
        // Err(Error::custom("top-level serializer supports only struct"))
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::custom("top-level serializer supports only struct"))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::custom("top-level serializer supports only struct"))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::custom("top-level serializer supports only struct"))
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::custom("top-level serializer supports only struct"))
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::custom("top-level serializer supports only struct"))
    }
}

impl<'input, 'output, T> ser::SerializeStruct for TopLevelSerializer<'input, 'output, T>
where
    T: 'output + Target,
{
    type Ok = &'output mut form_urlencoded::Serializer<'input, T>;
    type Error = Error;

    fn serialize_field<U>(
        &mut self,
        key: &'static str,
        value: &U,
    ) -> std::result::Result<(), Self::Error>
    where
        U: ?Sized + Serialize,
    {
        match &self.state {
            State::Init => Err(Error::custom("unexpected field")),
            State::WaitingForKey => {
                let serializer = TopLevelSerializer {
                    encoder: self.encoder,
                    state: State::WaitingForValue(Cow::Borrowed(key)),
                };
                value.serialize(serializer)?;
                Ok(())
            }
            State::WaitingForChildKey(parent_key) => {
                // Concatenate parent-key and child-key
                let key = format!("{}.{}", parent_key, key);
                let serializer = TopLevelSerializer {
                    encoder: self.encoder,
                    state: State::WaitingForValue(Cow::Owned(key)),
                };
                value.serialize(serializer)?;
                Ok(())
            }
            State::WaitingForValue(_) => Err(Error::custom("unexpected key and value")),
        }
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(self.encoder)
    }
}
