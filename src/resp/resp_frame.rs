use bytes::BytesMut;
use enum_dispatch::enum_dispatch;

use crate::{
    array::RespArray, bulk_string::BulkString, err::RespError, null::RespNull, set::RespSet,
    simple_error::SimpleError, simple_string::SimpleString, RespDecode,
};

use super::map::RespMap;

/// RESP(Redis serialization protocol specification).
/// According to https://redis.io/docs/latest/develop/reference/protocol-spec/.
#[enum_dispatch(RespEncode)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum RespFrame {
    SimpleString(SimpleString),
    Error(SimpleError),
    Null(RespNull),
    Integer(i64),
    BulkString(BulkString),
    Array(RespArray),
    Boolean(bool),
    Double(f64),
    Map(RespMap),
    Set(RespSet),
}

impl RespDecode for RespFrame {
    const PREFIX: &'static str = "";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        if buf.len() < 3 {
            return Err(RespError::NotCompleted);
        }
        let first = buf[0];
        let res: RespFrame = match first {
            b'+' => SimpleString::decode(buf)?.into(),
            b'-' => SimpleError::decode(buf)?.into(),
            b':' => i64::decode(buf)?.into(),
            b',' => f64::decode(buf)?.into(),
            b'#' => bool::decode(buf)?.into(),
            b'_' => RespNull::decode(buf)?.into(),
            b'$' => BulkString::decode(buf)?.into(),
            b'*' => RespArray::decode(buf)?.into(),
            b'%' => RespMap::decode(buf)?.into(),
            b'~' => RespSet::decode(buf)?.into(),
            _ => {
                return Err(RespError::InvalidFrameType(format!(
                    "unknown type: {}",
                    first
                )))
            }
        };
        Ok(res)
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let mut iter = buf.iter().peekable();
        match iter.peek() {
            Some(b'*') => RespArray::expect_length(buf),
            Some(b'~') => RespSet::expect_length(buf),
            Some(b'%') => RespMap::expect_length(buf),
            Some(b'#') => bool::expect_length(buf),
            Some(b':') => i64::expect_length(buf),
            Some(b',') => f64::expect_length(buf),
            Some(b'+') => SimpleString::expect_length(buf),
            Some(b'-') => SimpleError::expect_length(buf),
            Some(b'_') => RespNull::expect_length(buf),
            Some(b'$') => BulkString::expect_length(buf),
            _ => Err(RespError::NotCompleted),
        }
    }
}

impl From<&[u8]> for RespFrame {
    fn from(value: &[u8]) -> Self {
        BulkString(value.to_vec()).into()
    }
}

impl<const N: usize> From<&[u8; N]> for RespFrame {
    fn from(value: &[u8; N]) -> Self {
        BulkString(value.to_vec()).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resp_frame_decode() -> anyhow::Result<()> {
        // TODO: deal with null bulk string with bulk string
        // let mut buf = BytesMut::from("$-1\r\n");
        // let _result = RespFrame::decode(&mut buf)?;
        // assert_eq!(result, RespFrame::NullBulkString(RespNullBulkString));

        let mut buf = BytesMut::from("$5\r\nhello\r\n");
        let expected_length = RespFrame::expect_length(&buf)?;
        assert_eq!(expected_length, 11);
        let result = RespFrame::decode(&mut buf)?;
        assert_eq!(result, RespFrame::BulkString(b"hello".into()));
        Ok(())
    }
}
