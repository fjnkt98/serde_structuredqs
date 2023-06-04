use crate::de::{
    deserializer::Deserializer,
    error::{Error, Result},
    level::Level,
    utility::replace_plus,
};

use serde::de::Error as _;

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::iter::Iterator;
use std::slice::Iter;
use std::str;

pub struct Parser<'a> {
    inner: &'a [u8],
    iter: Iter<'a, u8>,
    index: usize,
    acc: (usize, usize),
    peeked: Option<&'a u8>,
    depth: i32,
    state: ParsingState,
}

enum ParsingState {
    Init,
    Key,
    Value,
}

impl<'a> Iterator for Parser<'a> {
    type Item = &'a u8;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.peeked.take() {
            Some(v) => Some(v),
            None => {
                self.index += 1;
                self.acc.1 += 1;
                self.iter.next()
            }
        }
    }
}

impl<'a> Parser<'a> {
    #[inline]
    fn peek(&mut self) -> Option<<Self as Iterator>::Item> {
        if self.peeked.is_some() {
            self.peeked
        } else if let Some(x) = self.next() {
            self.peeked = Some(x);
            Some(x)
        } else {
            None
        }
    }
}

impl<'a> Parser<'a> {
    pub fn new(encoded: &'a [u8]) -> Self {
        Parser {
            inner: encoded,
            iter: encoded.iter(),
            acc: (0, 0),
            index: 0,
            peeked: None,
            depth: 0,
            state: ParsingState::Init,
        }
    }

    fn clear_acc(&mut self) {
        self.acc = (self.index, self.index);
    }

    fn collect_str(&mut self) -> Result<Cow<'a, str>> {
        let replaced = replace_plus(&self.inner[self.acc.0..self.acc.1 - 1]);
        let decoder = percent_encoding::percent_decode(&replaced);

        let maybe_decoded = decoder
            .decode_utf8()
            .map_err(|e| Error::custom(e.to_string()))?;

        let ret: Result<Cow<'a, str>> = match maybe_decoded {
            Cow::Borrowed(_) => match replaced {
                Cow::Borrowed(_) => {
                    let res = str::from_utf8(&self.inner[self.acc.0..self.acc.1 - 1])
                        .map_err(|e| Error::custom(e.to_string()))?;
                    Ok(Cow::Borrowed(res))
                }
                Cow::Owned(owned) => {
                    let res = String::from_utf8(owned).map_err(|e| Error::custom(e.to_string()))?;
                    Ok(Cow::Owned(res))
                }
            },
            Cow::Owned(owned) => Ok(Cow::Owned(owned)),
        };
        self.clear_acc();
        ret.map_err(Error::from)
    }

    pub(crate) fn as_deserializer(&mut self) -> Result<Deserializer<'a>> {
        let map = BTreeMap::default();
        let mut root = Level::Nested(map);

        while self.parse(&mut root)? {}
        let iter = match root {
            Level::Nested(map) => map.into_iter(),
            _ => BTreeMap::default().into_iter(),
        };
        Ok(Deserializer { iter, value: None })
    }

    fn parse(&mut self, node: &mut Level<'a>) -> Result<bool> {
        // First character determines parsing type
        match self.next() {
            Some(x) => {
                match *x {
                    b'.' => loop {
                        self.clear_acc();
                        let key = self.parse_key(b'.', false)?;
                        self.parse_map_value(key, node)?;
                        return Ok(true);
                    },
                    b'&' => {
                        self.clear_acc();
                        Ok(true)
                    }
                    _ => {
                        let key = { self.parse_key(b'.', false)? };
                        // Root keys are _always_ map values
                        self.parse_map_value(key, node)?;
                        Ok(true)
                    }
                }
            }
            None => Ok(false),
        }
    }

    fn parse_key(&mut self, end_on: u8, consume: bool) -> Result<Cow<'a, str>> {
        self.state = ParsingState::Key;
        loop {
            if let Some(x) = self.peek() {
                if *x == b'=' {
                    return self.collect_str();
                }
            }
            if let Some(x) = self.next() {
                match *x {
                    c if end_on == c => {
                        if !consume {
                            self.peeked = Some(x);
                        }
                        return self.collect_str();
                    }
                    b'=' => {
                        if end_on != b']' {
                            self.peeked = Some(x);
                            return self.collect_str();
                        }
                    }
                    b'&' => {
                        self.peeked = Some(&b'&');
                        return self.collect_str();
                    }
                    _ => {
                        // for any other character
                        // do nothing, keep adding to key
                    }
                }
            } else {
                // no more string to parse
                return self.collect_str();
            }
        }
    }

    /// The `(key,value)` pair is determined to be corresponding to a map entry,
    /// so parse it as such. The first part of the `key` has been parsed.
    fn parse_map_value(&mut self, key: Cow<'a, str>, node: &mut Level<'a>) -> Result<()> {
        self.state = ParsingState::Key;
        let res = loop {
            if let Some(x) = self.peek() {
                match *x {
                    b'=' => {
                        // Key is finished, parse up until the '&' as the value
                        self.clear_acc();
                        self.state = ParsingState::Value;
                        for _ in self.take_while(|b| *b != &b'&') {}
                        let value: Cow<'a, str> = self.collect_str()?;
                        node.insert_map_value(key, value);
                        break Ok(());
                    }
                    b'&' => {
                        // No value
                        node.insert_map_value(key, Cow::Borrowed(""));
                        break Ok(());
                    }
                    b'.' => {
                        // The key continues to another level of nested.
                        // Add a new unitialised level for this node and continue.
                        if let Level::UnInitialized = *node {
                            *node = Level::Nested(BTreeMap::default());
                        }
                        if let Level::Nested(ref mut map) = *node {
                            // By parsing we drop down another level
                            self.depth -= 1;
                            // Either take the existing entry, or add a new
                            // unitialised level
                            // Use this new node to keep parsing
                            let _ = self.parse(map.entry(key).or_insert(Level::UnInitialized))?;
                            break Ok(());
                        } else {
                            // We expected to parse into a map here.
                            break Err(Error::custom(format!(
                                "tried to insert a \
                                     new key into {:?}",
                                node
                            )));
                        }
                    }
                    _ => {
                        // Anything else is unexpected since we just finished
                        // parsing a key.
                        let _ = self.next();
                    }
                }
            } else {
                // The string has ended, so the value is empty.
                node.insert_map_value(key, Cow::Borrowed(""));
                break Ok(());
            }
        };
        // We have finished parsing this level, so go back up a level.
        self.depth += 1;
        res
    }
}
