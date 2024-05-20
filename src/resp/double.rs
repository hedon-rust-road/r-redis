use bytes::BytesMut;

use crate::{err::RespError, extract_simple_frame_data, RespDecode, RespEncode, CRLF_LEN};

/// The Double RESP type encodes a double-precision floating point value.
/// Format:
///     ,[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n
///
/// - The comma character (,) as the first byte.
/// - An optional plus (+) or minus (-) as the sign.
/// - One or more decimal digits (0..9) as an unsigned, base-10 integral value.
/// - An optional dot (.), followed by one or more decimal digits (0..9) as an unsigned, base-10 fractional value.
/// - An optional capital or lowercase letter E (E or e),
///     followed by an optional plus (+) or minus (-) as the exponent's sign,
///     ending with one or more decimal digits (0..9) as an unsigned, base-10 exponent value.
/// - The CRLF terminator.
///
/// Example:
///     1.23
///     ,1.23\r\n
///
/// Other examples:
///     ,inf\r\n
///     ,-inf\r\n
///     ,nan\r\n
impl RespEncode for f64 {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(32);
        let ret = if self.abs() > 1e+8 || self.abs() < 1e-8 {
            format!(",{:e}\r\n", self)
        } else {
            let sign = if self < 0.0 || self.is_nan() { "" } else { "+" };
            format!(",{}{}\r\n", sign, self)
        };
        let ret = ret.to_lowercase();
        buf.extend_from_slice(&ret.into_bytes());
        buf
    }
}

impl RespDecode for f64 {
    const PREFIX: &'static str = ",";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        let content = buf.split_to(end + CRLF_LEN);
        let content = String::from_utf8_lossy(&content[Self::PREFIX.len()..end]).to_string();
        Ok(content.parse::<f64>()?)
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}

#[cfg(test)]
mod tests {
    use std::f64::{INFINITY, NAN, NEG_INFINITY};

    use crate::resp_frame::RespFrame;

    use super::*;

    #[test]
    fn test_double_encode() {
        let frame: RespFrame = (1.22).into();
        assert_eq!(frame.encode(), b",+1.22\r\n");
        let frame: RespFrame = (-1.22).into();
        assert_eq!(frame.encode(), b",-1.22\r\n");
        let frame: RespFrame = (0.0).into();
        assert_eq!(frame.encode(), b",0e0\r\n");
        let frame: RespFrame = (0.00000).into();
        assert_eq!(frame.encode(), b",0e0\r\n");
        let frame: RespFrame = (1.22e-10).into();
        assert_eq!(frame.encode(), b",1.22e-10\r\n");
        let frame: RespFrame = (1.22e+10).into();
        assert_eq!(frame.encode(), b",1.22e10\r\n");
        let frame: RespFrame = (INFINITY).into();
        assert_eq!(frame.encode(), b",inf\r\n");
        let frame: RespFrame = (-INFINITY).into();
        assert_eq!(frame.encode(), b",-inf\r\n");
        let frame: RespFrame = (NAN).into();
        assert_eq!(frame.encode(), b",nan\r\n");
    }

    #[test]
    fn test_double_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::from(",1.2\r\n");
        let result = f64::decode(&mut buf)?;
        assert_eq!(result, 1.2);

        let mut buf = BytesMut::from(",-1.2\r\n");
        let result = f64::decode(&mut buf)?;
        assert_eq!(result, -1.2);

        let mut buf = BytesMut::from(",inf\r\n");
        let result = f64::decode(&mut buf)?;
        assert_eq!(result, INFINITY);

        let mut buf = BytesMut::from(",-inf\r\n");
        let result = f64::decode(&mut buf)?;
        assert_eq!(result, NEG_INFINITY);

        let mut buf = BytesMut::from(",nan\r\n");
        let result = f64::decode(&mut buf)?;
        assert!(result.is_nan());

        let mut buf = BytesMut::from(",0e0\r\n");
        let result = f64::decode(&mut buf)?;
        assert_eq!(result, 0.0);

        let mut buf = BytesMut::from(",1.23e2\r\n");
        let result = f64::decode(&mut buf)?;
        assert_eq!(result, 123.0);

        let mut buf = BytesMut::from(",1.23e-2\r\n");
        let result = f64::decode(&mut buf)?;
        assert_eq!(result, 0.0123);

        let mut buf = BytesMut::from(",1.23e-10\r\n");
        let result = f64::decode(&mut buf)?;
        assert_eq!(result, 1.23e-10);

        let mut buf = BytesMut::from(",1.23e+10\r\n");
        let result = f64::decode(&mut buf)?;
        assert_eq!(result, 1.23e+10);
        Ok(())
    }
}
