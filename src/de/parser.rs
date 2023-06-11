use crate::{
    de::{deserializer::Deserializer, level::Level},
    error::{Error, Result},
};

use serde::de::Error as _;

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::iter::Iterator;
use std::slice::Iter;
use std::str;

pub(crate) fn replace_plus(input: &[u8]) -> Cow<[u8]> {
    match input.iter().position(|&b| b == b'+') {
        None => Cow::Borrowed(input),
        Some(first_position) => {
            let mut replaced = input.to_owned();
            replaced[first_position] = b' ';
            for byte in &mut replaced[first_position + 1..] {
                if *byte == b'+' {
                    *byte = b' ';
                }
            }

            Cow::Owned(replaced)
        }
    }
}

/// Parse x-www-form-urlencoded string into structured key-value mappings.
pub struct Parser<'a> {
    inner: &'a [u8],
    iter: Iter<'a, u8>,
    head: isize,
    tail: usize,
}

impl<'a> Iterator for Parser<'a> {
    type Item = &'a u8;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.head += 1;
        self.iter.next()
    }
}

impl<'a> Parser<'a> {
    pub fn new(encoded: &'a [u8]) -> Self {
        Parser {
            inner: encoded,
            iter: encoded.iter(),
            head: -1, // In the initial state, the head will start at index -1 to ensure that the head is 0 when iter() is first called.
            tail: 0,
        }
    }

    /// Shrink the range from the tail to the head.
    /// The tail will be positioned one after the head.
    fn shrink(&mut self) {
        self.tail = self.head as usize + 1;
    }

    /// Collect the URL-decoded string slice from the tail to the head with decoding and shrink the tail.
    fn collect_str(&mut self) -> Result<Cow<'a, str>> {
        let replaced = replace_plus(&self.inner[self.tail..self.head as usize]);
        let decoder = percent_encoding::percent_decode(&replaced);

        let maybe_decoded = decoder
            .decode_utf8()
            .map_err(|e| Error::custom(e.to_string()))?;

        let ret: Result<Cow<'a, str>> = match maybe_decoded {
            Cow::Borrowed(_) => match replaced {
                Cow::Borrowed(_) => {
                    let res = str::from_utf8(&self.inner[self.tail..self.head as usize])
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
        self.shrink();
        ret.map_err(Error::from)
    }

    /// Parse the entire input string into a Level struct, construct a Deserializer, and return it.
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

    /// The top-level parsing function. It checks the first character to determine the type of key
    /// and call the parsing function.
    fn parse(&mut self, node: &mut Level<'a>) -> Result<bool> {
        // Check the first character to determine parsing type
        match self.next() {
            Some(x) => {
                match *x {
                    // Set the tail position and parse key and value, which is the child of current node.
                    b'.' => {
                        self.shrink();
                        let key = self.parse_key()?;
                        self.parse_map_value(key, node)?;
                        return Ok(true);
                    }
                    // Set the tail position and continue to next.
                    b'&' => {
                        self.shrink();
                        Ok(true)
                    }
                    // Normally parse key and value.
                    _ => {
                        let key = self.parse_key()?;
                        // Root keys are _always_ map values
                        self.parse_map_value(key, node)?;
                        Ok(true)
                    }
                }
            }
            None => Ok(false),
        }
    }

    /// Collect the string slice up to the separator character and return the collected key.
    fn parse_key(&mut self) -> Result<Cow<'a, str>> {
        loop {
            if let Some(x) = self.next() {
                match *x {
                    b'.' | b'=' | b'&' => {
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
        let res = loop {
            // Check the character that current head is pointing to.
            match self.inner[self.head as usize] {
                b'=' => {
                    // Key is finished, parse up until the '&' as the value.
                    self.shrink();
                    for _ in self.take_while(|b| *b != &b'&') {}
                    let value: Cow<'a, str> = self.collect_str()?;
                    node.insert_map_value(key, value);
                    break Ok(());
                }
                b'&' => {
                    // There is no value, so insert empty string.
                    node.insert_map_value(key, Cow::Borrowed(""));
                    break Ok(());
                }
                b'.' => {
                    // The next string is a key of the nested map.

                    // If the node is uninitialized, initialize it with empty BTreeMap.
                    if let Level::UnInitialized = *node {
                        *node = Level::Nested(BTreeMap::default());
                    }
                    // if the node is the map, call the top-level parse function with the node.
                    if let Level::Nested(ref mut map) = *node {
                        self.shrink();
                        let _ = self.parse(map.entry(key).or_insert(Level::UnInitialized))?;
                        break Ok(());
                    } else {
                        // We expected to parse into a map here.
                        break Err(Error::custom(format!(
                            "tried to insert a new key into {:?}",
                            node
                        )));
                    }
                }
                _ => {
                    // Break the loop when the iterator is finished.
                    if let None = self.next() {
                        break Ok(());
                    }
                }
            }
        };

        res
    }
}
