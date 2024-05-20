use bytes::BytesMut;

use crate::{err::RespError, extract_simple_frame_data, RespDecode, RespEncode, CRLF_LEN};

pub const BOOL_LEN: usize = "#f\r\n".len();

/// #<t|f>\r\n
impl RespEncode for bool {
    fn encode(self) -> Vec<u8> {
        format!("#{}\r\n", if self { "t" } else { "f" }).into_bytes()
    }
}

impl RespDecode for bool {
    const PREFIX: &'static str = "#";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        let content = buf.split_to(end + CRLF_LEN);
        let content = String::from_utf8_lossy(&content[Self::PREFIX.len()..end]).to_string();
        if content == "t" {
            Ok(true)
        } else if content == "f" {
            Ok(false)
        } else {
            Err(RespError::InvalidFrameType(format!(
                "expected: #t or #f, got: {:?}",
                buf
            )))
        }
    }

    fn expect_length(_: &[u8]) -> Result<usize, RespError> {
        Ok(BOOL_LEN)
    }
}

#[cfg(test)]
mod tests {
    use crate::resp_frame::RespFrame;

    use super::*;

    #[test]
    fn test_boolean_encode() {
        let frame: RespFrame = true.into();
        assert_eq!(frame.encode(), b"#t\r\n");
        let frame: RespFrame = false.into();
        assert_eq!(frame.encode(), b"#f\r\n");
    }

    #[test]
    fn test_boolean_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::from("#t\r\n");
        let result = bool::decode(&mut buf)?;
        assert!(result);

        let mut buf = BytesMut::from("#f\r\n");
        let result = bool::decode(&mut buf)?;
        assert!(!result);

        let mut buf = BytesMut::from("#xxx\r\n");
        let result = bool::decode(&mut buf);
        assert!(result.is_err());
        Ok(())
    }
}
