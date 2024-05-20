use bytes::BytesMut;

use crate::{err::RespError, extract_simple_frame_data, RespDecode, RespEncode, CRLF_LEN};

/// This type is a CRLF-terminated string that represents a signed, base-10, 64-bit integer.
///
/// Format:
///     :[<+|->]<value>\r\n
///
/// - The colon (:) as the first byte.
/// - An optional plus (+) or minus (-) as the sign.
/// - One or more decimal digits (0..9) as the integer's unsigned, base-10 value.
/// - The CRLF terminator.
impl RespEncode for i64 {
    fn encode(self) -> Vec<u8> {
        format!(":{}\r\n", self).into_bytes()
    }
}

impl RespDecode for i64 {
    const PREFIX: &'static str = ":";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        let content = buf.split_to(end + CRLF_LEN);
        let content = String::from_utf8_lossy(&content[Self::PREFIX.len()..end]).to_string();
        Ok(content.parse::<i64>()?)
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}

#[cfg(test)]
mod tests {
    use bytes::BufMut;

    use crate::resp_frame::RespFrame;

    use super::*;

    #[test]
    fn test_integer_encode() {
        let frame: RespFrame = 0.into();
        assert_eq!(frame.encode(), b":0\r\n");
        let frame: RespFrame = (-123).into();
        assert_eq!(frame.encode(), b":-123\r\n");
        let frame: RespFrame = (123).into();
        assert_eq!(frame.encode(), b":123\r\n");
    }

    #[test]
    fn test_integer_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::from(":10\r\n");
        let result = i64::decode(&mut buf)?;
        assert_eq!(result, 10);

        buf.extend_from_slice(b":-10\r\n");
        let result = i64::decode(&mut buf)?;
        assert_eq!(result, -10);

        buf.extend_from_slice(b":0\r\n");
        let result = i64::decode(&mut buf)?;
        assert_eq!(result, 0);

        buf.extend_from_slice(b":100\r");
        let result = i64::decode(&mut buf);
        assert_eq!(result.unwrap_err(), RespError::NotCompleted);

        buf.put_u8(b'\n');
        let result = i64::decode(&mut buf)?;
        assert_eq!(result, 100);

        buf.extend_from_slice(b":xxx\r\n");
        let result = i64::decode(&mut buf);
        assert_eq!(
            result.unwrap_err().to_string(),
            "Parse int error: invalid digit found in string".to_string()
        );
        Ok(())
    }
}
