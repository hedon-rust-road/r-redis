use std::ops::Deref;

use bytes::BytesMut;

use crate::{
    cal_total_length, err::RespError, parse_length, parse_length_and_move, resp_frame::RespFrame,
    RespDecode, RespEncode, BUF_CAP,
};

pub const NULL_ARRAY: &[u8] = b"*-1\r\n";

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespArray(pub(crate) Option<Vec<RespFrame>>);

impl RespDecode for RespArray {
    const PREFIX: &'static str = "*";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        if buf.len() < Self::expect_length(buf)? {
            return Err(RespError::NotCompleted);
        }
        let length = parse_length_and_move(Self::PREFIX, buf)?;
        if length == -1 {
            return Ok(RespArray::null());
        }
        let mut array = Vec::with_capacity(length as usize);
        for _ in 0..length {
            let item = RespFrame::decode(buf)?;
            array.push(item);
        }
        Ok(RespArray::new(array))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(Self::PREFIX, buf)?;
        if len == -1 {
            return Ok(NULL_ARRAY.len());
        }
        cal_total_length(buf, end, len as usize, Self::PREFIX)
    }
}

/// Clients send commands to the Redis server as RESP arrays.
/// Similarly, some Redis commands that return collections of
/// elements use arrays as their replies.
///
/// Format:
///     *<number-of-elements>\r\n<element-1>...<element-n>
///
/// - An asterisk (*) as the first byte.
/// - One or more decimal digits (0..9) as the number of elements in the array as an unsigned, base-10 value.
/// - The CRLF terminator.
/// - An additional RESP type for every element of the array.
impl RespEncode for RespArray {
    fn encode(self) -> Vec<u8> {
        match self.0 {
            None => NULL_ARRAY.to_vec(),
            Some(v) => {
                let mut buf = Vec::with_capacity(BUF_CAP);
                buf.extend_from_slice(format!("*{}\r\n", v.len()).as_bytes());
                for frame in v {
                    buf.extend_from_slice(&frame.encode())
                }
                buf
            }
        }
    }
}

impl RespArray {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(Some(s.into()))
    }
    pub fn null() -> Self {
        RespArray(None)
    }
}

impl Deref for RespArray {
    type Target = Vec<RespFrame>;
    fn deref(&self) -> &Self::Target {
        static EMPTY_ARRAY: Vec<RespFrame> = vec![];
        self.0.as_ref().unwrap_or(&EMPTY_ARRAY)
    }
}

#[cfg(test)]
mod tests {
    use crate::{simple_error::SimpleError, simple_string::SimpleString};

    use super::*;

    #[test]
    fn test_array_decode() -> anyhow::Result<()> {
        // empty array
        let mut buf = BytesMut::from("*0\r\n");
        let result = RespArray::decode(&mut buf)?;
        assert_eq!(result, RespArray::new(vec![]));

        // one item
        let mut buf = BytesMut::from("*1\r\n+foo\r\n");
        let result = RespArray::decode(&mut buf)?;
        assert_eq!(
            result,
            RespArray::new(vec![SimpleString::new("foo").into()])
        );

        // many items, but in one type
        let mut buf = BytesMut::from("*2\r\n+foo\r\n+bar\r\n");
        let result = RespArray::decode(&mut buf)?;
        assert_eq!(
            result,
            RespArray::new(vec![
                SimpleString::new("foo").into(),
                SimpleString::new("bar").into()
            ])
        );

        // many items, but in different types
        let mut buf = BytesMut::from("*2\r\n+foo\r\n:1\r\n");
        let result = RespArray::decode(&mut buf)?;
        assert_eq!(
            result,
            RespArray::new(vec![SimpleString::new("foo").into(), (1).into()])
        );

        // not completed
        let mut buf = BytesMut::from("*2\r\n+foo\r\n");
        let result = RespArray::decode(&mut buf);
        assert_eq!(result.unwrap_err(), RespError::NotCompleted);

        // add bytes to buf to make it completed
        buf.extend_from_slice(b"+bar\r\n");
        let result = RespArray::decode(&mut buf)?;
        assert_eq!(
            result,
            RespArray::new(vec![
                SimpleString::new("foo").into(),
                SimpleString::new("bar").into()
            ])
        );

        // null array
        let mut buf = BytesMut::from("*-1\r\n");
        let result = RespArray::decode(&mut buf)?;
        assert_eq!(result, RespArray::null());
        Ok(())
    }

    #[test]
    fn test_array_encode() {
        let frame: RespFrame = RespArray::new(vec![
            SimpleString::new("hello").into(),
            SimpleError::new("Err").into(),
            123.into(),
        ])
        .into();
        assert_eq!(frame.encode(), b"*3\r\n+hello\r\n-Err\r\n:123\r\n");

        let frame: RespFrame = RespArray::null().into();
        assert_eq!(frame.encode(), b"*-1\r\n");
    }
}
