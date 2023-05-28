use crate::{
    error::{Error, Result},
    ser::part::Sink,
};
use serde::Serialize;
use std::{borrow::Cow, ops::Deref};

pub enum Key<'key> {
    Static(&'static str),
    Dynamic(Cow<'key, str>),
}

impl<'key> Deref for Key<'key> {
    type Target = str;

    fn deref(&self) -> &str {
        match *self {
            Key::Static(key) => key,
            Key::Dynamic(ref key) => key,
        }
    }
}

impl<'key> From<Key<'key>> for Cow<'static, str> {
    fn from(key: Key<'key>) -> Self {
        match key {
            Key::Static(key) => key.into(),
            Key::Dynamic(key) => key.into_owned().into(),
        }
    }
}

pub struct KeySink<End> {
    end: End,
}

impl<End, Ok> KeySink<End>
where
    End: for<'key> FnOnce(Key<'key>) -> Result<Ok>,
{
    pub fn new(end: End) -> Self {
        Self { end }
    }
}

impl<End, Ok> Sink for KeySink<End>
where
    End: for<'key> FnOnce(Key<'key>) -> Result<Ok>,
{
    type Ok = Ok;

    fn serialize_static_str(self, value: &'static str) -> Result<Ok> {
        (self.end)(Key::Static(value))
    }

    fn serialize_str(self, value: &str) -> Result<Ok> {
        (self.end)(Key::Dynamic(value.into()))
    }

    fn serialize_string(self, value: String) -> Result<Ok> {
        (self.end)(Key::Dynamic(value.into()))
    }

    fn serialize_none(self) -> Result<Ok> {
        Err(self.unsupported())
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Ok>
    where
        T: ?Sized + Serialize,
    {
        Err(self.unsupported())
    }

    fn unsupported(self) -> Error {
        Error::Custom("unsupported key".into())
    }
}
