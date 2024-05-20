use bytes::{Buf, BytesMut};

use crate::{resp::err::RespError, resp_frame::RespFrame, simple_string::SimpleString, RespDecode};

pub const CRLF: &[u8] = b"\r\n";
pub const CRLF_LEN: usize = CRLF.len();
pub const BOOL_LEN: usize = "#f\r\n".len();
pub const NULL_BULK_STRING: &[u8] = b"$-1\r\n";
pub const NULL: &[u8] = b"_\r\n";
pub const NULL_ARRAY: &[u8] = b"*-1\r\n";

pub fn extract_fixed_data(
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

pub fn extract_simple_frame_data(buf: &[u8], prefix: &str) -> Result<usize, RespError> {
    if buf.len() <= 3 {
        return Err(RespError::NotCompleted);
    }

    if !buf.starts_with(prefix.as_bytes()) {
        return Err(RespError::InvalidFrameType(format!(
            "expected: prefix ({}), got: {:?}",
            prefix, buf
        )));
    }

    if let Some(end) = find_crlf(buf, 1) {
        Ok(end)
    } else {
        Err(RespError::NotCompleted)
    }
}

/// nth starts from 1.
fn find_crlf(buf: &[u8], nth: i32) -> Option<usize> {
    let mut count = nth;
    (0..buf.len() - 1).find(|&i| {
        if buf[i] == b'\r' && buf[i + 1] == b'\n' {
            count -= 1;
            count == 0
        } else {
            false
        }
    })
}

pub fn parse_length(prefix: &str, buf: &[u8]) -> Result<(usize, usize), RespError> {
    let end = extract_simple_frame_data(buf, prefix)?;
    let length = String::from_utf8_lossy(&buf[prefix.len()..end]).to_string();
    let length = length.parse()?;
    Ok((end, length))
}

pub fn parse_length_and_move(prefix: &str, buf: &mut BytesMut) -> Result<usize, RespError> {
    let (end, length) = parse_length(prefix, buf)?;
    buf.advance(end + CRLF_LEN);
    Ok(length)
}

pub fn cal_total_length(
    buf: &[u8],
    end: usize,
    len: usize,
    prefix: &str,
) -> Result<usize, RespError> {
    let mut total: usize = end + CRLF_LEN;
    let mut data = &buf[total..];
    match prefix {
        "*" | "~" => {
            for _ in 0..len {
                let item_len = RespFrame::expect_length(data)?;
                data = &data[item_len..];
                total += item_len;
            }
            Ok(total)
        }
        "%" => {
            for _ in 0..len {
                let key_len = SimpleString::expect_length(data)?;
                data = &data[key_len..];
                total += key_len;

                let value_len = RespFrame::expect_length(data)?;
                data = &data[value_len..];
                total += value_len;
            }
            Ok(total)
        }
        _ => Ok(len + CRLF_LEN),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_crlf() {
        let buf = b"+\r\n";
        assert_eq!(find_crlf(buf, 0), None);
        assert_eq!(find_crlf(buf, 1), Some(1));
        assert_eq!(find_crlf(buf, 2), None);

        let buf = b"\r\nxxxx\r\naaa\r\n";
        assert_eq!(find_crlf(buf, 0), None);
        assert_eq!(find_crlf(buf, 1), Some(0));
        assert_eq!(find_crlf(buf, 2), Some(6));
        assert_eq!(find_crlf(buf, 3), Some(11));
    }
}
