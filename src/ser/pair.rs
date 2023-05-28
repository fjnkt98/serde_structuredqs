use crate::{
    error::{Error, Result},
    ser::{key::KeySink, part::PartSerializer, value::ValueSink},
};
use serde::{
    de::Error as _,
    ser::{self, SerializeTuple},
    Serialize,
};
use std::{borrow::Cow, mem};

pub struct PairSerializer<'input, 'target, Target: form_urlencoded::Target> {
    encoder: &'target mut form_urlencoded::Serializer<'input, Target>,
    state: PairState,
}

impl<'input, 'target, Target> PairSerializer<'input, 'target, Target>
where
    Target: 'target + form_urlencoded::Target,
{
    pub fn new(encoder: &'target mut form_urlencoded::Serializer<'input, Target>) -> Self {
        Self {
            encoder,
            state: PairState::WaitingForKey,
        }
    }
}

enum PairState {
    WaitingForKey,
    WaitingForValue { key: Cow<'static, str> },
    Done,
}

impl<'input, 'target, Target> ser::Serializer for PairSerializer<'input, 'target, Target>
where
    Target: 'target + form_urlencoded::Target,
{
    type Ok = ();
    type Error = Error;
    type SerializeSeq = ser::Impossible<(), Error>; // あとで対応する
    type SerializeTuple = Self;
    type SerializeTupleStruct = ser::Impossible<(), Error>;
    type SerializeTupleVariant = ser::Impossible<(), Error>;
    type SerializeMap = ser::Impossible<(), Error>;
    type SerializeStruct = ser::Impossible<(), Error>;
    type SerializeStructVariant = ser::Impossible<(), Error>;

    fn serialize_bool(self, _v: bool) -> Result<()> {
        Err(Error::custom("unsupported pair"))
    }
    fn serialize_i8(self, _v: i8) -> Result<Self::Ok> {
        Err(Error::custom("unsupported pair"))
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok> {
        Err(Error::custom("unsupported pair"))
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok> {
        Err(Error::custom("unsupported pair"))
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok> {
        Err(Error::custom("unsupported pair"))
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok> {
        Err(Error::custom("unsupported pair"))
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok> {
        Err(Error::custom("unsupported pair"))
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok> {
        Err(Error::custom("unsupported pair"))
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok> {
        Err(Error::custom("unsupported pair"))
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok> {
        Err(Error::custom("unsupported pair"))
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok> {
        Err(Error::custom("unsupported pair"))
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok> {
        Err(Error::custom("unsupported pair"))
    }

    fn serialize_str(self, _value: &str) -> Result<Self::Ok> {
        Err(Error::custom("unsupported pair"))
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<Self::Ok> {
        Err(Error::custom("unsupported pair"))
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Err(Error::custom("unsupported pair"))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        Err(Error::custom("unsupported pair"))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        Err(Error::custom("unsupported pair"))
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
        Err(Error::custom("unsupported pair"))
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        // あとで対応する
        Err(Error::custom("unsupported pair"))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self> {
        if len == 2 {
            Ok(self)
        } else {
            Err(Error::custom("unsupported pair"))
        }
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::custom("unsupported pair"))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::custom("unsupported pair"))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::custom("unsupported pair"))
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(Error::custom("unsupported pair"))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::custom("unsupported pair"))
    }
}

impl<'input, 'target, Target> SerializeTuple for PairSerializer<'input, 'target, Target>
where
    Target: 'target + form_urlencoded::Target,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match mem::replace(&mut self.state, PairState::Done) {
            PairState::WaitingForKey => {
                let key_sink = KeySink::new(|key| Ok(key.into()));
                let key_serializer = PartSerializer::new(key_sink);
                self.state = PairState::WaitingForValue {
                    key: value.serialize(key_serializer)?,
                };
                Ok(())
            }
            PairState::WaitingForValue { key } => {
                let result = {
                    let value_sink = ValueSink::new(self.encoder, &key);
                    let value_serializer = PartSerializer::new(value_sink);
                    value.serialize(value_serializer)
                };
                if result.is_ok() {
                    self.state = PairState::Done;
                } else {
                    self.state = PairState::WaitingForValue { key };
                }
                result
            }
            PairState::Done => Err(Error::custom("this pair has already been serialized")),
        }
    }

    fn end(self) -> Result<()> {
        if let PairState::Done = self.state {
            Ok(())
        } else {
            Err(Error::custom("this pair has not yet been serialized"))
        }
    }
}
