use crate::error::{Error, Result};
use form_urlencoded::Target;
use serde::{de::Error as _, ser, Serialize};
use std::borrow::Cow;

pub struct SeqSerializer<'input, 'output, T>
where
    T: Target,
{
    encoder: &'output mut form_urlencoded::Serializer<'input, T>,
    key: Cow<'static, str>,
    container: Vec<Cow<'input, str>>,
}

impl<'input, 'output, T> SeqSerializer<'input, 'output, T>
where
    T: 'output + Target,
{
    pub fn new(
        encoder: &'output mut form_urlencoded::Serializer<'input, T>,
        key: Cow<'static, str>,
        len: Option<usize>,
    ) -> Self {
        Self {
            encoder,
            key,
            container: Vec::with_capacity(len.unwrap_or(0)),
        }
    }
}

impl<'input, 'output, T> ser::SerializeSeq for SeqSerializer<'input, 'output, T>
where
    T: Target,
{
    type Ok = &'output mut form_urlencoded::Serializer<'input, T>;
    type Error = Error;

    fn serialize_element<S>(&mut self, value: &S) -> Result<()>
    where
        S: Serialize + ?Sized,
    {
        value.serialize(self)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(self
            .encoder
            .append_pair(&self.key, &self.container.join(",")))
    }
}

macro_rules! serialize_integer {
    ($ty:ty, $method:ident) => {
        fn $method(self, value: $ty) -> Result<Self::Ok> {
            let mut buf = itoa::Buffer::new();
            let value = buf.format(value);
            self.container.push(Cow::Owned(value.to_owned()));
            Ok(())
        }
    };
}

macro_rules! serialize_float {
    ($ty:ty, $method:ident) => {
        fn $method(self, value: $ty) -> Result<Self::Ok> {
            let mut buf = ryu::Buffer::new();
            let value = buf.format(value);
            self.container.push(Cow::Owned(value.to_owned()));
            Ok(())
        }
    };
}

impl<'input, 'output, T> ser::Serializer for &mut SeqSerializer<'input, 'output, T>
where
    T: 'output + Target,
{
    type Ok = ();
    type Error = Error;
    type SerializeSeq = ser::Impossible<Self::Ok, Error>;
    type SerializeTuple = ser::Impossible<Self::Ok, Error>;
    type SerializeTupleStruct = ser::Impossible<Self::Ok, Error>;
    type SerializeTupleVariant = ser::Impossible<Self::Ok, Error>;
    type SerializeMap = ser::Impossible<Self::Ok, Error>;
    type SerializeStruct = ser::Impossible<Self::Ok, Error>;
    type SerializeStructVariant = ser::Impossible<Self::Ok, Error>;

    serialize_integer!(i8, serialize_i8);
    serialize_integer!(i16, serialize_i16);
    serialize_integer!(i32, serialize_i32);
    serialize_integer!(i64, serialize_i64);
    serialize_integer!(u8, serialize_u8);
    serialize_integer!(u16, serialize_u16);
    serialize_integer!(u32, serialize_u32);
    serialize_integer!(u64, serialize_u64);
    serialize_float!(f32, serialize_f32);
    serialize_float!(f64, serialize_f64);

    fn serialize_bool(self, value: bool) -> Result<Self::Ok> {
        self.container
            .push(Cow::Borrowed(if value { "true" } else { "false" }));
        Ok(())
    }
    fn serialize_char(self, value: char) -> Result<Self::Ok> {
        self.container.push(Cow::Owned(value.to_string()));
        Ok(())
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        self.container.push(Cow::Owned(value.to_owned()));
        Ok(())
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_some<U>(self, value: &U) -> Result<Self::Ok>
    where
        U: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_struct<U>(self, _name: &'static str, _value: &U) -> Result<Self::Ok>
    where
        U: ?Sized + Serialize,
    {
        Err(Error::custom("value only supports primitive"))
    }
    fn serialize_bytes(self, _value: &[u8]) -> Result<Self::Ok> {
        Err(Error::custom("value only supports primitive"))
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        Err(Error::custom("value only supports primitive"))
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
        Err(Error::custom("value only supports primitive"))
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::custom("value only supports primitive"))
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::custom("value only supports primitive"))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::custom("value only supports primitive"))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::custom("value only supports primitive"))
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::custom("value only supports primitive"))
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::custom("value only supports primitive"))
    }
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(Error::custom("value only supports primitive"))
    }
}
