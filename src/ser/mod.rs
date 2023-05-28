mod key;
mod pair;
mod part;
mod value;

use crate::{
    error::{Error, Result},
    ser::{part::PartSerializer, value::ValueSink},
};
use serde::{de::Error as _, ser, Serialize};
use std::borrow::Cow;

pub struct Serializer<'input, 'output, Target: form_urlencoded::Target> {
    encoder: &'output mut form_urlencoded::Serializer<'input, Target>,
    state: KeyState,
}

enum KeyState {
    Empty,
    WaitingForKey,
    WaitingForChildKey { parent_key: Cow<'static, str> },
    WaitingForValue { key: Cow<'static, str> },
}

impl<'input, 'output, Target: form_urlencoded::Target> Serializer<'input, 'output, Target> {
    pub fn new(encoder: &'output mut form_urlencoded::Serializer<'input, Target>) -> Self {
        Serializer {
            encoder,
            state: KeyState::Empty,
        }
    }
}

pub fn to_string<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    let mut encoder = form_urlencoded::Serializer::new(String::from(""));
    value.serialize(Serializer::new(&mut encoder))?;
    Ok(encoder.finish())
}

impl<'input, 'output, Target> ser::Serializer for Serializer<'input, 'output, Target>
where
    Target: 'output + form_urlencoded::Target,
{
    type Ok = &'output mut form_urlencoded::Serializer<'input, Target>;
    type Error = Error;

    type SerializeSeq = ser::Impossible<Self::Ok, Error>;
    type SerializeTuple = ser::Impossible<Self::Ok, Error>;
    type SerializeTupleStruct = ser::Impossible<Self::Ok, Error>;
    type SerializeTupleVariant = ser::Impossible<Self::Ok, Error>;
    type SerializeMap = ser::Impossible<Self::Ok, Error>;
    type SerializeStruct = Self;
    type SerializeStructVariant = ser::Impossible<Self::Ok, Error>;

    fn serialize_bool(self, value: bool) -> Result<Self::Ok> {
        match self.state {
            KeyState::Empty => Err(Error::custom("top level serializer supports only structs")),
            KeyState::WaitingForKey => Err(Error::custom("key not found")),
            KeyState::WaitingForChildKey { parent_key: _ } => {
                Err(Error::custom("unexpected value"))
            }
            KeyState::WaitingForValue { key } => {
                let value_sink = ValueSink::new(self.encoder, &key);
                let value_serializer = PartSerializer::new(value_sink);
                value.serialize(value_serializer)?;
                Ok(self.encoder)
            }
        }
    }

    fn serialize_i8(self, value: i8) -> Result<Self::Ok> {
        match self.state {
            KeyState::Empty => Err(Error::custom("top level serializer supports only structs")),
            KeyState::WaitingForKey => Err(Error::custom("key not found")),
            KeyState::WaitingForChildKey { parent_key: _ } => {
                Err(Error::custom("unexpected value"))
            }
            KeyState::WaitingForValue { key } => {
                let value_sink = ValueSink::new(self.encoder, &key);
                let value_serializer = PartSerializer::new(value_sink);
                value.serialize(value_serializer)?;
                Ok(self.encoder)
            }
        }
    }

    fn serialize_i16(self, value: i16) -> Result<Self::Ok> {
        match self.state {
            KeyState::Empty => Err(Error::custom("top level serializer supports only structs")),
            KeyState::WaitingForKey => Err(Error::custom("key not found")),
            KeyState::WaitingForChildKey { parent_key: _ } => {
                Err(Error::custom("unexpected value"))
            }
            KeyState::WaitingForValue { key } => {
                let value_sink = ValueSink::new(self.encoder, &key);
                let value_serializer = PartSerializer::new(value_sink);
                value.serialize(value_serializer)?;
                Ok(self.encoder)
            }
        }
    }

    fn serialize_i32(self, value: i32) -> Result<Self::Ok> {
        match self.state {
            KeyState::Empty => Err(Error::custom("top level serializer supports only structs")),
            KeyState::WaitingForKey => Err(Error::custom("key not found")),
            KeyState::WaitingForChildKey { parent_key: _ } => {
                Err(Error::custom("unexpected value"))
            }
            KeyState::WaitingForValue { key } => {
                let value_sink = ValueSink::new(self.encoder, &key);
                let value_serializer = PartSerializer::new(value_sink);
                value.serialize(value_serializer)?;
                Ok(self.encoder)
            }
        }
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok> {
        match self.state {
            KeyState::Empty => Err(Error::custom("top level serializer supports only structs")),
            KeyState::WaitingForKey => Err(Error::custom("key not found")),
            KeyState::WaitingForChildKey { parent_key: _ } => {
                Err(Error::custom("unexpected value"))
            }
            KeyState::WaitingForValue { key } => {
                let value_sink = ValueSink::new(self.encoder, &key);
                let value_serializer = PartSerializer::new(value_sink);
                value.serialize(value_serializer)?;
                Ok(self.encoder)
            }
        }
    }

    fn serialize_u8(self, value: u8) -> Result<Self::Ok> {
        match self.state {
            KeyState::Empty => Err(Error::custom("top level serializer supports only structs")),
            KeyState::WaitingForKey => Err(Error::custom("key not found")),
            KeyState::WaitingForChildKey { parent_key: _ } => {
                Err(Error::custom("unexpected value"))
            }
            KeyState::WaitingForValue { key } => {
                let value_sink = ValueSink::new(self.encoder, &key);
                let value_serializer = PartSerializer::new(value_sink);
                value.serialize(value_serializer)?;
                Ok(self.encoder)
            }
        }
    }

    fn serialize_u16(self, value: u16) -> Result<Self::Ok> {
        match self.state {
            KeyState::Empty => Err(Error::custom("top level serializer supports only structs")),
            KeyState::WaitingForKey => Err(Error::custom("key not found")),
            KeyState::WaitingForChildKey { parent_key: _ } => {
                Err(Error::custom("unexpected value"))
            }
            KeyState::WaitingForValue { key } => {
                let value_sink = ValueSink::new(self.encoder, &key);
                let value_serializer = PartSerializer::new(value_sink);
                value.serialize(value_serializer)?;
                Ok(self.encoder)
            }
        }
    }

    fn serialize_u32(self, value: u32) -> Result<Self::Ok> {
        match self.state {
            KeyState::Empty => Err(Error::custom("top level serializer supports only structs")),
            KeyState::WaitingForKey => Err(Error::custom("key not found")),
            KeyState::WaitingForChildKey { parent_key: _ } => {
                Err(Error::custom("unexpected value"))
            }
            KeyState::WaitingForValue { key } => {
                let value_sink = ValueSink::new(self.encoder, &key);
                let value_serializer = PartSerializer::new(value_sink);
                value.serialize(value_serializer)?;
                Ok(self.encoder)
            }
        }
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok> {
        match self.state {
            KeyState::Empty => Err(Error::custom("top level serializer supports only structs")),
            KeyState::WaitingForKey => Err(Error::custom("key not found")),
            KeyState::WaitingForChildKey { parent_key: _ } => {
                Err(Error::custom("unexpected value"))
            }
            KeyState::WaitingForValue { key } => {
                let value_sink = ValueSink::new(self.encoder, &key);
                let value_serializer = PartSerializer::new(value_sink);
                value.serialize(value_serializer)?;
                Ok(self.encoder)
            }
        }
    }

    fn serialize_f32(self, value: f32) -> Result<Self::Ok> {
        match self.state {
            KeyState::Empty => Err(Error::custom("top level serializer supports only structs")),
            KeyState::WaitingForKey => Err(Error::custom("key not found")),
            KeyState::WaitingForChildKey { parent_key: _ } => {
                Err(Error::custom("unexpected value"))
            }
            KeyState::WaitingForValue { key } => {
                let value_sink = ValueSink::new(self.encoder, &key);
                let value_serializer = PartSerializer::new(value_sink);
                value.serialize(value_serializer)?;
                Ok(self.encoder)
            }
        }
    }

    fn serialize_f64(self, value: f64) -> Result<Self::Ok> {
        match self.state {
            KeyState::Empty => Err(Error::custom("top level serializer supports only structs")),
            KeyState::WaitingForKey => Err(Error::custom("key not found")),
            KeyState::WaitingForChildKey { parent_key: _ } => {
                Err(Error::custom("unexpected value"))
            }
            KeyState::WaitingForValue { key } => {
                let value_sink = ValueSink::new(self.encoder, &key);
                let value_serializer = PartSerializer::new(value_sink);
                value.serialize(value_serializer)?;
                Ok(self.encoder)
            }
        }
    }

    fn serialize_char(self, value: char) -> Result<Self::Ok> {
        match self.state {
            KeyState::Empty => Err(Error::custom("top level serializer supports only structs")),
            KeyState::WaitingForKey => Err(Error::custom("key not found")),
            KeyState::WaitingForChildKey { parent_key: _ } => {
                Err(Error::custom("unexpected value"))
            }
            KeyState::WaitingForValue { key } => {
                let value_sink = ValueSink::new(self.encoder, &key);
                let value_serializer = PartSerializer::new(value_sink);
                value.serialize(value_serializer)?;
                Ok(self.encoder)
            }
        }
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        match self.state {
            KeyState::Empty => Err(Error::custom("top level serializer supports only structs")),
            KeyState::WaitingForKey => Err(Error::custom("key not found")),
            KeyState::WaitingForChildKey { parent_key: _ } => {
                Err(Error::custom("unexpected value"))
            }
            KeyState::WaitingForValue { key } => {
                let value_sink = ValueSink::new(self.encoder, &key);
                let value_serializer = PartSerializer::new(value_sink);
                value.serialize(value_serializer)?;
                Ok(self.encoder)
            }
        }
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<Self::Ok> {
        Err(Error::custom("top level serializer supports only structs"))
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(self.encoder)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        Ok(self.encoder)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        Err(Error::custom("top level serializer supports only structs"))
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::custom("top level serializer supports only structs"))
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Ok(self.encoder)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::custom("top level serializer supports only structs"))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::custom("top level serializer supports only structs"))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::custom("top level serializer supports only structs"))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::custom("top level serializer supports only structs"))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::custom("top level serializer supports only structs"))
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        match self.state {
            KeyState::Empty => {
                // トップレベルのstructを処理するとき
                Ok(Self {
                    encoder: self.encoder,
                    state: KeyState::WaitingForKey,
                })
            }
            KeyState::WaitingForChildKey { parent_key: _ } => {
                Err(Error::custom("unexpected state"))
            }
            KeyState::WaitingForKey => Err(Error::custom("the key has not yet provided")),
            KeyState::WaitingForValue { key } => {
                // structを値に持つフィールドを処理するとき
                Ok(Self {
                    encoder: self.encoder,
                    state: KeyState::WaitingForChildKey { parent_key: key },
                })
            }
        }
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::custom("top level serializer supports only structs"))
    }
}

impl<'input, 'output, Target> ser::SerializeStruct for Serializer<'input, 'output, Target>
where
    Target: 'output + form_urlencoded::Target,
{
    type Ok = &'output mut form_urlencoded::Serializer<'input, Target>;
    type Error = Error;

    fn serialize_field<V>(&mut self, key: &'static str, value: &V) -> Result<()>
    where
        V: ?Sized + Serialize,
    {
        match &self.state {
            KeyState::Empty => Err(Error::custom("unexpected field")),
            KeyState::WaitingForKey => {
                let serializer = Serializer {
                    encoder: self.encoder,
                    state: KeyState::WaitingForValue {
                        key: Cow::Borrowed(key),
                    },
                };
                value.serialize(serializer)?;
                Ok(())
            }
            KeyState::WaitingForChildKey { parent_key } => {
                let key = format!("{}.{}", parent_key, key);
                let serializer = Serializer {
                    encoder: self.encoder,
                    state: KeyState::WaitingForValue {
                        key: Cow::Owned(key),
                    },
                };
                value.serialize(serializer)?;
                Ok(())
            }
            KeyState::WaitingForValue { key: _ } => Err(Error::custom("unexpected key and value")),
        }
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(self.encoder)
    }
}
