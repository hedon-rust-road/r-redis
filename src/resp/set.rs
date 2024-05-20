use std::ops::Deref;

use bytes::BytesMut;

use crate::{
    decode::{cal_total_length, parse_length, parse_length_and_move},
    encode::BUF_CAP,
    err::RespError,
    resp_frame::RespFrame,
    RespDecode, RespEncode,
};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespSet(Vec<RespFrame>);

/// Sets are somewhat like Arrays but are unordered and should only contain unique elements.
/// Format:
///     ~<number-of-elements>\r\n<element-1>...<element-n>
///
/// - A tilde (~) as the first byte.
/// - One or more decimal digits (0..9) as the number of elements in the set as an unsigned, base-10 value.
/// - The CRLF terminator.
/// - An additional RESP type for every element of the Set.
impl RespEncode for RespSet {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("~{}\r\n", self.len()).into_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.encode());
        }
        buf
    }
}

impl RespDecode for RespSet {
    const PREFIX: &'static str = "~";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        if buf.len() < Self::expect_length(buf)? {
            return Err(RespError::NotCompleted);
        }
        let length = parse_length_and_move(Self::PREFIX, buf)?;
        let mut data = Vec::with_capacity(length);
        for _ in 0..length {
            let key = RespFrame::decode(buf)?;
            if data.contains(&key) {
                continue;
            }
            data.push(key);
        }
        Ok(RespSet::new(data))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(Self::PREFIX, buf)?;
        cal_total_length(buf, end, len, Self::PREFIX)
    }
}

impl RespSet {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespSet(s.into())
    }
}

impl Deref for RespSet {
    type Target = Vec<RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::simple_string::SimpleString;

    use super::*;

    #[test]
    fn test_set_encode() {
        let set = RespSet::new(vec![1.into(), 2.into()]);
        let frame: RespFrame = set.into();
        assert_eq!(frame.encode(), b"~2\r\n:1\r\n:2\r\n");
    }

    #[test]
    fn test_set_decode() -> anyhow::Result<()> {
        // empty set
        let mut buf = BytesMut::from("~0\r\n");
        let result = RespSet::decode(&mut buf)?;
        assert_eq!(result, RespSet::new(vec![]));

        // one item
        let mut buf = BytesMut::from("~1\r\n+foo\r\n");
        let result = RespSet::decode(&mut buf)?;
        assert_eq!(result, RespSet::new(vec![SimpleString::new("foo").into()]));

        // many items in one type
        let mut buf = BytesMut::from("~2\r\n+foo\r\n+bar\r\n");
        let result = RespSet::decode(&mut buf)?;
        assert_eq!(
            result,
            RespSet::new(vec![
                SimpleString::new("foo").into(),
                SimpleString::new("bar").into()
            ])
        );

        // many items in different types
        let mut buf = BytesMut::from("~2\r\n+foo\r\n:1\r\n");
        let result = RespSet::decode(&mut buf)?;
        assert_eq!(
            result,
            RespSet::new(vec![SimpleString::new("foo").into(), (1).into()])
        );

        // has duplicated items
        let mut buf = BytesMut::from("~2\r\n+foo\r\n+foo\r\n");
        let result = RespSet::decode(&mut buf)?;
        assert_eq!(result, RespSet::new(vec![SimpleString::new("foo").into()]));

        // not completed
        let mut buf = BytesMut::from("~2\r\n+foo\r\n");
        let result = RespSet::decode(&mut buf);
        assert_eq!(result.unwrap_err(), RespError::NotCompleted);

        // add bytes to buf to make it completed
        buf.extend_from_slice(b"+baz\r\n");
        let result = RespSet::decode(&mut buf)?;
        let expected = RespSet::new(vec![
            SimpleString::new("foo").into(),
            SimpleString::new("baz").into(),
        ]);
        assert_eq!(result, expected);
        Ok(())
    }
}
