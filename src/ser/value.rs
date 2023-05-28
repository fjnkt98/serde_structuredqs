use crate::{
    error::{Error, Result},
    ser::part::{PartSerializer, Sink},
};
use serde::Serialize;
use std::str;

pub struct ValueSink<'input, 'key, 'target, Target>
where
    Target: form_urlencoded::Target,
{
    encoder: &'target mut form_urlencoded::Serializer<'input, Target>,
    key: &'key str,
}

impl<'input, 'key, 'target, Target> ValueSink<'input, 'key, 'target, Target>
where
    Target: 'target + form_urlencoded::Target,
{
    pub fn new(
        encoder: &'target mut form_urlencoded::Serializer<'input, Target>,
        key: &'key str,
    ) -> Self {
        Self { encoder, key }
    }
}

impl<'input, 'key, 'target, Target> Sink for ValueSink<'input, 'key, 'target, Target>
where
    Target: 'target + form_urlencoded::Target,
{
    type Ok = ();

    fn serialize_str(self, value: &str) -> Result<()> {
        self.encoder.append_pair(self.key, value);
        Ok(())
    }

    fn serialize_static_str(self, value: &'static str) -> Result<()> {
        self.serialize_str(value)
    }

    fn serialize_string(self, value: String) -> Result<()> {
        self.serialize_str(&value)
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok> {
        value.serialize(PartSerializer::new(self))
    }

    fn unsupported(self) -> Error {
        Error::Custom("unsupported value".into())
    }
}
