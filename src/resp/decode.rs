/*
pub enum RespFrame {
    SimpleString(SimpleString),
    Error(SimpleError),

    NullBulkString(RespNullBulkString),
    NullArray(RespNullArray),
    Null(RespNull),

    Integer(i64),
    BulkString(BulkString),
    Array(RespArray),
    Boolean(bool),
    Double(f64),
    Map(RespMap),
    Set(RespSet),
}
 */

use bytes::BytesMut;

use crate::{err::RespError, RespDecode, SimpleString};

// +OK\r\n
impl RespDecode for SimpleString {
    const PREFIX: &'static str = "+";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        if buf.len() <= 3 {
            return Err(RespError::NotCompleted);
        }
        if !buf.starts_with(Self::PREFIX.as_bytes()) {
            return Err(RespError::InvalidFrameType(format!(
                "expected: SimpleString(+), got: {:?}",
                buf
            )));
        }

        // try to find \r\n.
        let mut end = 0;
        for i in 0..buf.len() - 1 {
            if buf[i] == b'\r' && buf[i + 1] == b'\n' {
                end = i;
                break;
            }
        }
        if end == 0 {
            return Err(RespError::NotCompleted);
        }

        // split the string content.
        let content = buf.split_to(end + 2);
        let content = String::from_utf8_lossy(&content[1..end]).to_string();

        Ok(SimpleString::new(content))
    }
}

#[cfg(test)]
mod tests {
    use bytes::BufMut;

    use super::*;
    use crate::{err::RespError, RespDecode};

    #[test]
    fn test_simple_string() -> anyhow::Result<()> {
        // successful case
        let mut buf = BytesMut::from("+OK\r\n");
        let result = SimpleString::decode(&mut buf)?;
        assert_eq!(result.0, "OK");

        // not completed case
        buf.extend_from_slice(b"+Hi\r");
        let result = SimpleString::decode(&mut buf);
        assert_eq!(result.unwrap_err(), RespError::NotCompleted);

        // put \n to complete the string.
        buf.put_u8(b'\n');
        let result = SimpleString::decode(&mut buf)?;
        assert_eq!(result.0, "Hi");

        Ok(())
    }
}
