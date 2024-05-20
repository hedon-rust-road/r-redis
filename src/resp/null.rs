use bytes::BytesMut;

use crate::{
    decode::{extract_fixed_data, NULL},
    err::RespError,
    RespDecode, RespEncode,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct RespNull;

impl RespDecode for RespNull {
    const PREFIX: &'static str = "_";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        extract_fixed_data(buf, NULL, "RespNull")?;
        Ok(RespNull)
    }

    fn expect_length(_: &[u8]) -> Result<usize, RespError> {
        Ok(NULL.len())
    }
}

/// The null data type represents non-existent values.
///
/// Examples: _\r\n
impl RespEncode for RespNull {
    fn encode(self) -> Vec<u8> {
        b"_\r\n".to_vec()
    }
}

#[cfg(test)]
mod tests {
    use bytes::BufMut;

    use crate::resp_frame::RespFrame;

    use super::*;

    #[test]
    fn test_null_decode() -> anyhow::Result<()> {
        // successful case
        let mut buf = BytesMut::from("_\r\n");
        let result = RespNull::decode(&mut buf)?;
        assert_eq!(result, RespNull);

        // not completed case
        buf.extend_from_slice(b"_\r");
        let result = RespNull::decode(&mut buf);
        assert_eq!(result.unwrap_err(), RespError::NotCompleted);

        // put \n to complete the string.
        buf.put_u8(b'\n');
        let result = RespNull::decode(&mut buf)?;
        assert_eq!(result, RespNull);

        Ok(())
    }

    #[test]
    fn test_null_encode() {
        let frame: RespFrame = RespNull.into();
        assert_eq!(frame.encode(), b"_\r\n");
    }
}
