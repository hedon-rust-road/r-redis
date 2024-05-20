use std::ops::Deref;

use bytes::{Buf, BytesMut};

use crate::{
    err::RespError, parse_length, parse_length_and_move, RespDecode, RespEncode, CRLF, CRLF_LEN,
};

pub const NULL_BULK_STRING: &[u8] = b"$-1\r\n";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct BulkString(pub(crate) bool, pub(crate) Vec<u8>);

/// A bulk string represents a single binary string.
/// The string can be of any size, but by default,
/// Redis limits it to 512 MB (see the proto-max-bulk-len configuration directive).
///
/// Format:
///     $<length>\r\n<data>\r\n
///
/// - The dollar sign ($) as the first byte.
/// - One or more decimal digits (0..9) as the string's length, in bytes, as an unsigned, base-10 value.
/// - The CRLF terminator.
/// - The data.
/// - A final CRLF.
impl RespEncode for BulkString {
    fn encode(self) -> Vec<u8> {
        if self.0 {
            return NULL_BULK_STRING.to_vec();
        }
        let mut buf = Vec::with_capacity(self.len() + 16);
        buf.extend_from_slice(&format!("${}\r\n", self.len()).into_bytes());
        buf.extend_from_slice(&self);
        buf.extend_from_slice(b"\r\n");
        buf
    }
}

impl RespDecode for BulkString {
    const PREFIX: &'static str = "$";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let length = parse_length_and_move(Self::PREFIX, buf)?;
        if length == -1 {
            return Ok(BulkString(true, vec![]));
        }
        if buf.len() < length as usize + CRLF_LEN {
            return Err(RespError::NotCompleted);
        }
        let content: BytesMut = buf.split_to(length as usize);
        if !buf.starts_with(CRLF) {
            return Err(RespError::InvalidFrameType(format!(
                "expected: CRLF, got: {:?}",
                buf
            )));
        }
        buf.advance(CRLF_LEN);
        Ok(BulkString::new(content))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, length) = parse_length(Self::PREFIX, buf)?;
        if length == -1 {
            Ok(NULL_BULK_STRING.len())
        } else {
            Ok(end + CRLF_LEN + length as usize + CRLF_LEN)
        }
    }
}

impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString(false, s.into())
    }

    pub fn new_null() -> Self {
        BulkString(true, vec![])
    }
}

impl Deref for BulkString {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl From<&str> for BulkString {
    fn from(value: &str) -> Self {
        BulkString(false, value.as_bytes().to_vec())
    }
}

impl From<&[u8]> for BulkString {
    fn from(value: &[u8]) -> Self {
        BulkString(false, value.to_vec())
    }
}

impl<const N: usize> From<&[u8; N]> for BulkString {
    fn from(value: &[u8; N]) -> Self {
        BulkString(false, value.to_vec())
    }
}

impl AsRef<[u8]> for BulkString {
    fn as_ref(&self) -> &[u8] {
        &self.1
    }
}

#[cfg(test)]
mod tests {
    use crate::resp_frame::RespFrame;

    use super::*;

    #[test]
    fn test_bulk_string_encode() {
        let frame: RespFrame = BulkString::new_null().into();
        assert_eq!(frame.encode(), b"$-1\r\n");
        let frame: RespFrame = BulkString::new(b"hello").into();
        assert_eq!(frame.encode(), b"$5\r\nhello\r\n");
    }

    #[test]
    fn test_bulk_string_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::from("$5\r\nhello\r\n");
        let result = BulkString::decode(&mut buf)?;
        assert!(!result.0);
        assert_eq!(result.1, b"hello");

        let mut buf = BytesMut::from("$5\r\nhell\r\n");
        let result = BulkString::decode(&mut buf);
        assert!(result.is_err());

        let mut buf = BytesMut::from("$-1\r\n");
        let result = BulkString::decode(&mut buf)?;
        assert!(result.0);
        assert_eq!(result.1, vec![]);
        Ok(())
    }
}
