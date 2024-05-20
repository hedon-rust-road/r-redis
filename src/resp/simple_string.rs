use bytes::BytesMut;

use crate::{err::RespError, extract_simple_frame_data, RespDecode, RespEncode, CRLF_LEN};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct SimpleString(pub(crate) String);

impl RespDecode for SimpleString {
    const PREFIX: &'static str = "+";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        let content = buf.split_to(end + CRLF_LEN);
        let content = String::from_utf8_lossy(&content[Self::PREFIX.len()..end]).to_string();
        Ok(SimpleString::new(content))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}

/// Simple strings are encoded as a plus (+) character, followed by a string.
/// The string mustn't contain a CR (\r) or LF (\n) character and is terminated by CRLF (i.e., \r\n).
///
/// Examples: +OK\r\n
impl RespEncode for SimpleString {
    fn encode(self) -> Vec<u8> {
        format!("+{}\r\n", self.0).into_bytes()
    }
}

impl SimpleString {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleString(s.into())
    }
}

impl From<&str> for SimpleString {
    fn from(s: &str) -> Self {
        SimpleString(s.to_string())
    }
}

impl AsRef<str> for SimpleString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use bytes::BufMut;

    use crate::resp_frame::RespFrame;

    use super::*;

    #[test]
    fn test_simple_string_decode() -> anyhow::Result<()> {
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

    #[test]
    fn test_simple_string_encode() {
        let frame: RespFrame = SimpleString::new("OK".to_string()).into();
        assert_eq!(frame.encode(), b"+OK\r\n");
        let frame: RespFrame = SimpleString::new("hello".to_string()).into();
        assert_eq!(frame.encode(), b"+hello\r\n");
    }

    #[test]
    fn test_bulk_string_expect_length() -> anyhow::Result<()> {
        // TODO: deal with null string with simple string.
        // let buf = BytesMut::from("$-1\r\n");
        // let len = RespFrame::expect_length(&buf)?;
        // assert_eq!(len, 5);

        let buf = BytesMut::from("$2\r\nhi\r\n");
        let len = RespFrame::expect_length(&buf)?;
        assert_eq!(len, 8);
        Ok(())
    }
}
