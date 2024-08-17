use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use bytes::BytesMut;

use crate::{
    cal_total_length, err::RespError, parse_length, parse_length_and_move, resp_frame::RespFrame,
    simple_string::SimpleString, RespDecode, RespEncode, BUF_CAP,
};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespMap(BTreeMap<String, RespFrame>);

/// The RESP map encodes a collection of key-value tuples, i.e., a dictionary or a hash.
/// Format:
///     %<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>
///
/// - A percent character (%) as the first byte.
/// - One or more decimal digits (0..9) as the number of entries, or key-value tuples, in the map as an unsigned, base-10 value.
/// - The CRLF terminator.
/// - Two additional RESP types for every key and value in the map.
///
/// Examples:
///     {
///         "first": 1,
///         "second": 2
///     }
///            ↓
///         %2\r\n
///         +first\r\n
///         :1\r\n
///         +second\r\n
///         :2\r\n
/// (The raw RESP encoding is split into multiple lines for readability).
impl RespEncode for RespMap {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("%{}\r\n", self.len()).into_bytes());
        for (key, value) in self.0 {
            buf.extend(SimpleString::new(key).encode());
            buf.extend(&value.encode());
        }
        buf
    }
}

impl RespDecode for RespMap {
    const PREFIX: &'static str = "%";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        if buf.len() < Self::expect_length(buf)? {
            return Err(RespError::NotCompleted);
        }
        let length = parse_length_and_move(Self::PREFIX, buf)?;
        let mut map = RespMap::new();
        for _ in 0..length {
            let key = SimpleString::decode(buf)?;
            let value = RespFrame::decode(buf)?;
            map.insert(key.0, value);
        }
        Ok(map)
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(Self::PREFIX, buf)?;
        cal_total_length(buf, end, len as usize, Self::PREFIX)
    }
}

impl RespMap {
    pub fn new() -> Self {
        RespMap(BTreeMap::new())
    }
}

impl Default for RespMap {
    fn default() -> Self {
        RespMap::new()
    }
}

impl Deref for RespMap {
    type Target = BTreeMap<String, RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RespMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<BTreeMap<String, RespFrame>> for RespMap {
    fn from(value: BTreeMap<String, RespFrame>) -> Self {
        RespMap(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_encode() {
        let mut map = RespMap::new();
        map.insert("first".to_string(), 1.into());
        map.insert("second".to_string(), 2.into());
        let frame: RespFrame = map.into();
        assert_eq!(frame.encode(), b"%2\r\n+first\r\n:1\r\n+second\r\n:2\r\n");
    }

    #[test]
    fn test_map_decode() -> anyhow::Result<()> {
        // empty map
        let mut buf = BytesMut::from("%0\r\n");
        let result = RespMap::decode(&mut buf)?;
        assert_eq!(result, RespMap::new());

        // one item
        let mut buf = BytesMut::from("%1\r\n+foo\r\n+bar\r\n");
        let result = RespMap::decode(&mut buf)?;
        let mut expected = RespMap::new();
        expected.insert("foo".to_string(), SimpleString::new("bar").into());
        assert_eq!(result, expected);

        // many items in one type
        let mut buf = BytesMut::from("%2\r\n+foo\r\n+bar\r\n+baz\r\n+qux\r\n");
        let result = RespMap::decode(&mut buf)?;
        let mut expected = RespMap::new();
        expected.insert("foo".to_string(), SimpleString::new("bar").into());
        expected.insert("baz".to_string(), SimpleString::new("qux").into());
        assert_eq!(result, expected);

        // many items in different types
        let mut buf = BytesMut::from("%2\r\n+foo\r\n+bar\r\n+baz\r\n:2\r\n");
        let result = RespMap::decode(&mut buf)?;
        let mut expected = RespMap::new();
        expected.insert("foo".to_string(), SimpleString::new("bar").into());
        expected.insert("baz".to_string(), (2).into());
        assert_eq!(result, expected);

        // not completed
        let mut buf = BytesMut::from("%2\r\n+foo\r\n+bar\r\n");
        let result = RespMap::decode(&mut buf);
        assert_eq!(result.unwrap_err(), RespError::NotCompleted);

        // add bytes to buf to make it completed
        buf.extend_from_slice(b"+baz\r\n+qux\r\n");
        let result = RespMap::decode(&mut buf)?;
        let mut expected = RespMap::new();
        expected.insert("foo".to_string(), SimpleString::new("bar").into());
        expected.insert("baz".to_string(), SimpleString::new("qux").into());
        assert_eq!(result, expected);
        Ok(())
    }
}
