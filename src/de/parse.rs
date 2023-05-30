use super::*;
use percent_encoding::{percent_encode, AsciiSet, NON_ALPHANUMERIC};
use serde::de;
use std::{borrow::Cow, collections::BTreeMap, iter::Iterator, slice::Iter, str};

pub const QS_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b' ')
    .remove(b'*')
    .remove(b'-')
    .remove(b'.')
    .remove(b'_');

pub fn replace_space(input: &str) -> Cow<str> {
    match input.as_bytes().iter().position(|&b| b == b' ') {
        None => Cow::Borrowed(input),
        Some(first_position) => {
            let mut replaced = input.as_bytes().to_owned();
            replaced[first_position] = b'+';
            for byte in &mut replaced[first_position + 1..] {
                if *byte == b' ' {
                    *byte = b'+';
                }
            }
            Cow::Owned(String::from_utf8(replaced).expect("replacing ' ' with '+' cannot panic"))
        }
    }
}

macro_rules! tu {
    ($x:expr) => {
        match $x {
            Some(x) => *x,
            None => return Err(Error::custom("query string ended before expected")),
        };
    };
}

impl<'a> Level<'a> {
    fn insert_map_value(&mut self, key: Cow<'a, str>, value: Cow<'a, str>) {
        if let Level::Nested(ref mut map) = *self {
            match map.entry(key) {
                Entry::Occupied(mut o) => {
                    let key = o.key();
                    let error = if key.contains('[') {
                        let newkey = percent_encode(key.as_bytes(), QS_ENCODE_SET)
                            .map(replace_space)
                            .collect::<String>();
                        format!("multiple values for one key {}", key)
                    } else {
                        format!("multiple values for one key {}", key)
                    };
                    let _ = o.insert(Level::Invalid(error));
                }
                Entry::Vacant(vm) => {
                    let _ = vm.insert(Level::Flat(value));
                }
            }
        } else if let Level::Uninitialized = *self {
            let mut map = BTreeMap::default();
            let _ = map.insert(key, Level::Flat(value));
            *self = Level::Nested(map);
        } else {
            *self =
                Level::Invalid("attempted to insert map value into non-map structure".to_string())
        }
    }

    fn insert_ord_seq_value(&mut self, key: usize, value: Cow<'a, str>) {
        if let Level::OrderedSeq(ref mut map) = *self {
            match map.entry(key) {
                Entry::Occupied(mut o) => {
                    let _ = o.insert(Level::Invalid("multiple values for one key".to_string()));
                }
                Entry::Vacant(vm) => {
                    let _ = vm.insert(Level::Flat(value));
                }
            }
        } else if let Level::Uninitialized = *self {
            let mut map = BTreeMap::default();
            let _ = map.insert(key, Level::Flat(value));
            *self = Level::OrderedSeq(map);
        } else {
            *self =
                Level::Invalid("attempted to insert seq value into non-seq structure".to_string())
        }
    }

    fn insert_seq_value(&mut self, value: Cow<'a, str>) {
        if let Level::Sequence(ref mut seq) = *self {
            seq.push(Level::Flat(value));
        } else if let Level::Uninitialized = *self {
            let seq = vec![Level::Flat(value)];
            *self = Level::Sequence(seq);
        } else {
            *self =
                Level::Invalid("attempte to insert seq value into non-seq structure".to_string())
        }
    }
}

pub struct Parser<'a> {
    inner: &'a [u8],
    iter: Iter<'a, u8>,
    index: usize,
    acc: (usize, usize),
    peeked: Option<&'a u8>,
    depth: usize,
    strict: bool,
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
        let preparse_brackets = match self.state {
            ParsingState::Value => false,
            _ => !self.strict,
        };

        if preparse_brackets {
            match self.peeked.take() {
                Some(v) => Some(v),
                None => {
                    self.index += 1;
                    self.acc.1 += 1;
                    match self.iter.next() {
                        Some(v) if v == &b'%' && self.iter.len() >= 2 => {
                            match &self.iter.as_slice()[..2] {
                                b"5B" => {
                                    let _ = self.iter.next();
                                    let _ = self.iter.next();
                                    self.index += 2;
                                    Some(&b'[')
                                }
                                b"5D" => {
                                    let _ = self.iter.next();
                                    let _ = self.iter.next();
                                    self.index += 2;
                                    Some(&b']')
                                }
                                _ => Some(v),
                            }
                        }
                        Some(v) => Some(v),
                        None => None,
                    }
                }
            }
        } else {
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

fn replace_plus(input: &[u8]) -> Cow<[u8]> {
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

impl<'a> Parser<'a> {
    pub fn new(encoded: &'a [u8], depth: usize, strict: bool) -> Self {
        Self {
            inner: encoded,
            iter: encoded.iter(),
            acc: (0, 0),
            index: 0,
            peeked: None,
            depth,
            strict,
            state: ParsingState::Init,
        }
    }

    fn clear_acc(&mut self) {
        self.acc = (self.index, self.index);
    }

    fn collect_str(&mut self) -> Result<Cow<'a, str>> {
        let replaced = replace_plus(&self.inner[self.acc.0..self.acc.1 - 1]);
        let decoder = percent_encoding::percent_decode(&replaced);

        let maybe_decoded = if self.strict {
            decoder.decode_utf8()?
        } else {
            decoder.decode_utf8_lossy()
        };

        let ret: Result<Cow<'a, str>> = match maybe_decoded {
            Cow::Borrowed(_) => match replaced {
                Cow::Borrowed(_) => {
                    let res = str::from_utf8(&self.inner[self.acc.0..self.acc.1 - 1])?;
                    Ok(Cow::Borrowed(res))
                }
                Cow::Owned(owned) => {
                    let res = String::from_utf8(owned)?;
                    Ok(Cow::Owned(res))
                }
            },
            Cow::Owned(owned) => Ok(Cow::Owned(owned)),
        };
        self.clear_acc();
        ret.map_err(Error::from)
    }

    pub(crate) fn as_deserializer(&mut self) -> Result<QsDeserializer<'a>> {
        let map = BTreeMap::default();
        let mut root = Level::Nested(map);

        while self.parse(&mut root)? {}
        let iter = match root {
            Level::Nested(map) => map.into_iter(),
            _ => BTreeMap::default().into_iter(),
        };
        Ok(QsDeserializer { iter, value: None })
    }

    fn parse(&mut self, node: &mut Level<'a>) -> Result<bool> {
        if self.depth == 0 {
            let key = self.parse_key(b'=', false)?;
            self.parse_map_value(key, node)?;
            return Ok(true);
        }

        match self.next() {
            Some(x) => match *x {
                b'[' => loop {
                    self.clear_acc();
                    match tu!(self.peek()) {
                        b'[' => {
                            if self.strict {
                                return Err(super::Error::parse_error(
                                    "found another opening bracket before the closed bracket",
                                    self.index,
                                ));
                            } else {
                                let _ = self.next();
                            }
                        }
                        b']' => {
                            let _ = self.next();
                            self.clear_acc();
                            self.parse_seq_value(node)?;
                            return Ok(true);
                        }
                        b'0'..=b'9' => {
                            let key = self.parse_key(b']', true)?;
                            let key = key.parse().map_err(Error::from)?;
                            self.parse_ord_seq_value(key, node)?;
                            return Ok(true);
                        }
                        0x20..=0x2f | 0x3a..=0x5a | 0x5c | 0x5e..=0x7e => {
                            let key = self.parse_key(b']', true)?;
                            self.parse_map_value(key, node)?;
                            return Ok(true);
                        }
                        c => {
                            if self.strict {
                                return Err(super::Error::parse_error(
                                    format!(
                                        "unexpected character: {}",
                                        String::from_utf8_lossy(&[c]),
                                    ),
                                    self.index,
                                ));
                            } else {
                                let _ = self.next();
                            }
                        }
                    }
                },
                b'&' => {
                    self.clear_acc();
                    Ok(true)
                }
                _ => {
                    let key = { self.parse_key(b'[', false)? };
                    self.parse_map_value(key, node)?;
                    Ok(true)
                }
            },
            None => Ok(false),
        }
    }

    fn parse_key(&mut self, end_on: u8, consume: bool) -> Result<Cow<'a, str>> {
        self.state = ParsingState::Key;
        loop {
            if let Some(x) = self.next() {
                match *x {
                    c if c == end_on => {
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
                    _ => {}
                }
            } else {
                return self.collect_str();
            }
        }
    }

    fn parse_map_value(&mut self, key: Cow<'a, str>, node: &mut Level<'a>) -> Result<()> {
        self.state = ParsingState::Key;
        let res = loop {
            if let Some(x) = self.peek() {
                match *x {
                    b'=' => {
                        self.clear_acc();
                        self.state = ParsingState::Value;
                        for _ in self.take_while(|b| *b != &b'&') {}
                        let value: Cow<'a, str> = self.collect_str()?;
                        node.insert_map_value(key, value);
                        break Ok(());
                    }
                    b'&' => {
                        node.insert_map_value(key, Cow::Borrowed(""));
                        break Ok(());
                    }
                    b'[' => {
                        if let Level::Uninitialized = *node {
                            *node = Level::Nested(BTreeMap::default());
                        }
                        if let Level::Nested(ref mut map) = *node {
                            self.depth -= 1;
                            let _ = self.parse(map.entry(key).or_insert(Level::Uninitialized))?;
                            break Ok(());
                        }
                    }
                    c => {
                        if self.strict {
                            break Err(super::Error::parse_error(
                                format!(
                                    "unexpected character: '{}' found when parsing",
                                    String::from_utf8_lossy(&[c])
                                ),
                                self.index,
                            ));
                        } else {
                            let _ = self.next();
                        }
                    }
                }
            } else {
                node.insert_map_value(key, Cow::Borrowed(""));
                break Ok(());
            }
        };
        self.depth += 1;
        res
    }

    fn parse_ord_seq_value(&mut self, key: usize, node: &mut Level<'a>) -> Result<()> {
        self.state = ParsingState::Key;
        let res = loop {
            if let Some(x) = self.peek() {
                match *x {
                    b'=' => {
                        self.clear_acc();
                        self.state = ParsingState::Value;
                        for _ in self.take_while(|b| *b != &b'&') {}
                        let value = self.collect_str()?;
                        node.insert_ord_seq_value(key, value);
                        break Ok(());
                    }
                    b'&' => {
                        node.insert_ord_seq_value(key, Cow::Borrowed(""));
                        break Ok(());
                    }
                    b'[' => {
                        if let Level::Uninitialized = *node {
                            *node = Level::OrderedSeq(BTreeMap::default());
                        }
                        if let Level::OrderedSeq(ref mut map) = *node {
                            self.depth -= 1;
                            let _ = self.parse(map.entry(key).or_insert(Level::Uninitialized))?;
                            break Ok(());
                        } else {
                            break Err(super::Error::parse_error(
                                format!("tried to insert a new key into {:?}", node),
                                self.index,
                            ));
                        }
                    }
                    c => {
                        if self.strict {
                            break Err(super::Error::parse_error(
                                format!("unexpected character: {:?} found when parsing", c),
                                self.index,
                            ));
                        } else {
                            let _ = self.next();
                        }
                    }
                }
            } else {
                node.insert_ord_seq_value(key, Cow::Borrowed(""));
                break Ok(());
            }
        };
        self.depth += 1;
        res
    }

    fn parse_seq_value(&mut self, node: &mut Level<'a>) -> Result<()> {
        self.state = ParsingState::Key;
        let res = match self.peek() {
            Some(x) => match *x {
                b'=' => {
                    self.clear_acc();
                    self.state = ParsingState::Value;
                    for _ in self.take_while(|b| *b != &b'&') {}
                    let value = self.collect_str()?;
                    node.insert_seq_value(value);
                    Ok(())
                }
                b'&' => {
                    node.insert_seq_value(Cow::Borrowed(""));
                    Ok(())
                }
                _ => Err(super::Error::parse_error(
                    "non-indexed sequence of structs not supported",
                    self.index,
                )),
            },
            None => {
                node.insert_seq_value(Cow::Borrowed(""));
                Ok(())
            }
        };
        self.depth += 1;
        res
    }
}
