use bytes::{Buf, BytesMut};

use crate::{
    err::RespError, BulkString, RespArray, RespDecode, RespFrame, RespMap, RespNull, RespNullArray,
    RespNullBulkString, RespSet, SimpleError, SimpleString,
};

const CRLF: &[u8] = b"\r\n";
const CRLF_LEN: usize = CRLF.len();

/**
pub enum RespFrame {
    Map(RespMap),
    Set(RespSet),
}
 */

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
            b'$' => match RespNullBulkString::decode(buf) {
                Ok(resp) => resp.into(),
                Err(RespError::NotCompleted) => return Err(RespError::NotCompleted),
                Err(_) => BulkString::decode(buf)?.into(),
            },
            b'*' => match RespNullArray::decode(buf) {
                Ok(resp) => resp.into(),
                Err(RespError::NotCompleted) => return Err(RespError::NotCompleted),
                Err(_) => RespArray::decode(buf)?.into(),
            },
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
}

impl RespDecode for SimpleString {
    const PREFIX: &'static str = "+";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        let content = buf.split_to(end + CRLF_LEN);
        let content = String::from_utf8_lossy(&content[Self::PREFIX.len()..end]).to_string();
        Ok(SimpleString::new(content))
    }
}

impl RespDecode for SimpleError {
    const PREFIX: &'static str = "-";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        let content = buf.split_to(end + CRLF_LEN);
        let content = String::from_utf8_lossy(&content[Self::PREFIX.len()..end]).to_string();
        Ok(SimpleError::new(content))
    }
}

impl RespDecode for RespNullBulkString {
    const PREFIX: &'static str = "$";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        const NULL_BULK_STRING: &[u8] = b"$-1\r\n";
        extract_fixed_data(buf, NULL_BULK_STRING, "RespNullBulkString")?;
        Ok(RespNullBulkString)
    }
}

impl RespDecode for RespNull {
    const PREFIX: &'static str = "_";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        const NULL: &[u8] = b"_\r\n";
        extract_fixed_data(buf, NULL, "RespNull")?;
        Ok(RespNull)
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
}

impl RespDecode for BulkString {
    const PREFIX: &'static str = "$";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let length = parse_length(Self::PREFIX, buf)?;
        if buf.len() < length + CRLF_LEN {
            return Err(RespError::NotCompleted);
        }
        let content = buf.split_to(length);
        if !buf.starts_with(CRLF) {
            return Err(RespError::InvalidFrameType(format!(
                "expected: CRLF, got: {:?}",
                buf
            )));
        }
        buf.advance(CRLF_LEN);
        Ok(BulkString::new(content))
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
}

impl RespDecode for f64 {
    const PREFIX: &'static str = ",";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        let content = buf.split_to(end + CRLF_LEN);
        let content = String::from_utf8_lossy(&content[Self::PREFIX.len()..end]).to_string();
        Ok(content.parse::<f64>()?)
    }
}

impl RespDecode for RespNullArray {
    const PREFIX: &'static str = "*";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        const NULL_ARRAY: &[u8] = b"*-1\r\n";
        extract_fixed_data(buf, NULL_ARRAY, "RespNullArray")?;
        Ok(RespNullArray)
    }
}

impl RespDecode for RespArray {
    const PREFIX: &'static str = "*";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let length = parse_length(Self::PREFIX, buf)?;
        let mut array = Vec::with_capacity(length);
        for _ in 0..length {
            let item = RespFrame::decode(buf)?;
            // TODO:If is not complete, we cannot advance the buf.
            array.push(item);
        }
        Ok(RespArray::new(array))
    }
}

impl RespDecode for RespMap {
    const PREFIX: &'static str = "%";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let length = parse_length(Self::PREFIX, buf)?;
        let mut map = RespMap::new();
        for _ in 0..length {
            let key = SimpleString::decode(buf)?;
            let value = RespFrame::decode(buf)?;
            map.insert(key.0, value);
        }
        Ok(map)
    }
}

impl RespDecode for RespSet {
    const PREFIX: &'static str = "~";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let length = parse_length(Self::PREFIX, buf)?;
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
}

fn extract_fixed_data(
    buf: &mut BytesMut,
    expect: &[u8],
    expect_type: &str,
) -> Result<(), RespError> {
    if buf.len() < expect.len() {
        return Err(RespError::NotCompleted);
    }
    if !buf.starts_with(expect) {
        return Err(RespError::InvalidFrameType(format!(
            "expected: {}, got: {:?}",
            expect_type, buf
        )));
    }

    buf.advance(expect.len());
    Ok(())
}

fn extract_simple_frame_data(buf: &[u8], prefix: &str) -> Result<usize, RespError> {
    if buf.len() <= 3 {
        return Err(RespError::NotCompleted);
    }

    if !buf.starts_with(prefix.as_bytes()) {
        return Err(RespError::InvalidFrameType(format!(
            "expected: prefix ({}), got: {:?}",
            prefix, buf
        )));
    }

    if let Some(end) = find_crlf(buf) {
        Ok(end)
    } else {
        Err(RespError::NotCompleted)
    }
}

fn find_crlf(buf: &[u8]) -> Option<usize> {
    (1..buf.len() - 1).find(|&i| buf[i] == b'\r' && buf[i + 1] == b'\n')
}

fn parse_length(prefix: &str, buf: &mut BytesMut) -> Result<usize, RespError> {
    let end = extract_simple_frame_data(buf, prefix)?;
    let length = buf.split_to(end + CRLF_LEN);
    let length = String::from_utf8_lossy(&length[prefix.len()..end]).to_string();
    let length = length.parse::<usize>()?;
    Ok(length)
}

#[cfg(test)]
mod tests {
    use std::f64::{INFINITY, NEG_INFINITY};

    use bytes::BufMut;

    use super::*;
    use crate::{err::RespError, RespDecode};

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
    fn test_null_bulk_string_decode() -> anyhow::Result<()> {
        // successful case
        let mut buf = BytesMut::from("$-1\r\n");
        let result = RespNullBulkString::decode(&mut buf)?;
        assert_eq!(result, RespNullBulkString);

        // not completed case
        buf.extend_from_slice(b"$-1\r");
        let result = RespNullBulkString::decode(&mut buf);
        assert_eq!(result.unwrap_err(), RespError::NotCompleted);

        // put \n to complete the string.
        buf.put_u8(b'\n');
        let result = RespNullBulkString::decode(&mut buf)?;
        assert_eq!(result, RespNullBulkString);

        Ok(())
    }

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

    #[test]
    fn test_bulk_string_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::from("$5\r\nhello\r\n");
        let result = BulkString::decode(&mut buf)?;
        assert_eq!(result.0, b"hello");

        let mut buf = BytesMut::from("$5\r\nhell\r\n");
        let result = BulkString::decode(&mut buf);
        assert!(result.is_err());
        Ok(())
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

    #[test]
    fn test_null_array_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::from("*-1\r\n");
        let result = RespNullArray::decode(&mut buf)?;
        assert_eq!(result, RespNullArray);
        Ok(())
    }

    #[test]
    fn test_array_encode() -> anyhow::Result<()> {
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
        // buf.extend_from_slice(b"+bar\r\n");
        // let result = RespArray::decode(&mut buf)?;
        // assert_eq!(
        //     result,
        //     RespArray::new(vec![
        //         SimpleString::new("foo").into(),
        //         SimpleString::new("bar").into()
        //     ])
        // );
        Ok(())
    }

    #[test]
    fn test_map_decode() -> anyhow::Result<()> {
        // empty map
        let mut buf = BytesMut::from("%0\r\n");
        let result = RespMap::decode(&mut buf)?;
        assert_eq!(result, RespMap::new());

        // one item
        let mut buf = BytesMut::from("%1\r\n+foo\r\n+bar\r\n");
        let result = RespMap::decode(&mut buf)?;
        let mut expected = RespMap::new();
        expected.insert("foo".to_string(), SimpleString::new("bar").into());
        assert_eq!(result, expected);

        // many items in one type
        let mut buf = BytesMut::from("%2\r\n+foo\r\n+bar\r\n+baz\r\n+qux\r\n");
        let result = RespMap::decode(&mut buf)?;
        let mut expected = RespMap::new();
        expected.insert("foo".to_string(), SimpleString::new("bar").into());
        expected.insert("baz".to_string(), SimpleString::new("qux").into());
        assert_eq!(result, expected);

        // many items in different types
        let mut buf = BytesMut::from("%2\r\n+foo\r\n+bar\r\n+baz\r\n:2\r\n");
        let result = RespMap::decode(&mut buf)?;
        let mut expected = RespMap::new();
        expected.insert("foo".to_string(), SimpleString::new("bar").into());
        expected.insert("baz".to_string(), (2).into());
        assert_eq!(result, expected);
        Ok(())
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
        Ok(())
    }

    // one item}
}
