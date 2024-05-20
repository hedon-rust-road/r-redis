use bytes::BytesMut;

use crate::{
    decode::{extract_simple_frame_data, CRLF_LEN},
    err::RespError,
    RespDecode, RespEncode,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct SimpleError(String);

impl RespDecode for SimpleError {
    const PREFIX: &'static str = "-";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        let content = buf.split_to(end + CRLF_LEN);
        let content = String::from_utf8_lossy(&content[Self::PREFIX.len()..end]).to_string();
        Ok(SimpleError::new(content))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}

/// Simple errors, or simply just errors, are similar to simple strings,
/// but their first character is the minus (-) character.
///
/// The difference between simple strings and errors in RESP is
/// that clients should treat errors as exceptions,
/// whereas the string encoded in the error type is the error message itself.
///
/// Examples: -Error message\r\n
impl RespEncode for SimpleError {
    fn encode(self) -> Vec<u8> {
        format!("-{}\r\n", self.0).into_bytes()
    }
}

impl SimpleError {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleError(s.into())
    }
}

impl From<&str> for SimpleError {
    fn from(value: &str) -> Self {
        SimpleError(value.to_string())
    }
}

impl From<String> for SimpleError {
    fn from(value: String) -> Self {
        SimpleError(value)
    }
}

#[cfg(test)]
mod tests {
    use bytes::BufMut;

    use crate::resp_frame::RespFrame;

    use super::*;

    #[test]
    fn test_simple_error_decode() -> anyhow::Result<()> {
        // successful case
        let mut buf = BytesMut::from("-Err\r\n");
        let result = SimpleError::decode(&mut buf)?;
        assert_eq!(result.0, "Err");

        // not completed case
        buf.extend_from_slice(b"-Hi\r");
        let result = SimpleError::decode(&mut buf);
        assert_eq!(result.unwrap_err(), RespError::NotCompleted);

        // put \n to complete the string.
        buf.put_u8(b'\n');
        let result = SimpleError::decode(&mut buf)?;
        assert_eq!(result.0, "Hi");

        Ok(())
    }

    #[test]
    fn test_simple_error_encode() {
        let frame: RespFrame = SimpleError::new("Error Message".to_string()).into();
        assert_eq!(frame.encode(), b"-Error Message\r\n");
    }
}
