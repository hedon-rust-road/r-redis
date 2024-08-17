use winnow::{
    ascii::{digit1, float},
    combinator::{alt, dispatch, fail, opt, preceded, terminated},
    error::{ContextError, ErrMode},
    token::{any, take, take_until},
    PResult, Parser,
};

use crate::{BulkString, RespArray, RespFrame, RespMap, RespNull, SimpleError, SimpleString};

const CRLF: &[u8] = b"\r\n";

pub fn parse_frame(input: &mut &[u8]) -> PResult<RespFrame> {
    dispatch!(any;
        b'+' => simple_string.map(RespFrame::SimpleString),
        b'-' => error.map(RespFrame::Error),
        b':' => integer.map(RespFrame::Integer),
        b'$' => alt((null_bulk_string.map(RespFrame::BulkString), bulk_string.map(RespFrame::BulkString))),
        b'*' => alt((null_array.map(RespFrame::Array), array.map(RespFrame::Array))),
        b'_' => null.map(RespFrame::Null),
        b'#' => boolean.map(RespFrame::Boolean),
        b',' => double.map(RespFrame::Double),
        b'%' => map.map(RespFrame::Map),
        _v => fail::<_,_,_>
    )
    .parse_next(input)
}

// +OK\r\n
fn simple_string(input: &mut &[u8]) -> PResult<SimpleString> {
    parse_string.map(SimpleString).parse_next(input)
}

// -Error message\r\n
fn error(input: &mut &[u8]) -> PResult<SimpleError> {
    parse_string.map(SimpleError).parse_next(input)
}

// :[<+|->]<value>\r\n
pub(crate) fn integer(input: &mut &[u8]) -> PResult<i64> {
    let sign = opt(alt(('+', '-'))).parse_next(input)?.unwrap_or('+');
    let sign = if sign == '+' { 1 } else { -1 };
    let v: i64 = terminated(digit1.parse_to(), CRLF).parse_next(input)?;
    Ok(sign * v)
}

// $-1\r\n null bulk string
fn null_bulk_string(input: &mut &[u8]) -> PResult<BulkString> {
    "-1\r\n".value(BulkString(None)).parse_next(input)
}

// $<length>\r\n<data>\r\n
#[allow(clippy::comparison_chain)]
fn bulk_string(input: &mut &[u8]) -> PResult<BulkString> {
    let len = integer.parse_next(input)?;
    if len == 0 {
        return Ok(BulkString(Some(vec![])));
    } else if len < 0 {
        return Err(cut_err("bulk string len < 0 is invalid"));
    }
    let data = terminated(take(len as usize), CRLF)
        .map(|s: &[u8]| s.to_vec())
        .parse_next(input)?;
    Ok(BulkString(Some(data)))
}

// *-1\r\n
fn null_array(input: &mut &[u8]) -> PResult<RespArray> {
    "-1\r\n".value(RespArray::null()).parse_next(input)
}

// *<number-of-elements>\r\n<element-1>...<element-n>
#[allow(clippy::comparison_chain)]
fn array(input: &mut &[u8]) -> PResult<RespArray> {
    let len = integer.parse_next(input)?;
    if len == 0 {
        return Ok(RespArray::new(vec![]));
    } else if len < 0 {
        return Err(cut_err("array len < 0 is invalid"));
    }
    let mut arr = Vec::with_capacity(len as usize);
    for _ in 0..len {
        arr.push(parse_frame(input)?);
    }
    Ok(RespArray::new(arr))
}

// _\r\n
fn null(input: &mut &[u8]) -> PResult<RespNull> {
    CRLF.value(RespNull).parse_next(input)
}

// #<t|f>\r\n
fn boolean(input: &mut &[u8]) -> PResult<bool> {
    let b = alt(("t\r\n", "f\r\n")).parse_next(input)?;
    Ok(b[0] == b't')
}

fn double(input: &mut &[u8]) -> PResult<f64> {
    terminated(float, CRLF).parse_next(input)
}

fn map(input: &mut &[u8]) -> PResult<RespMap> {
    let len: i64 = integer.parse_next(input)?;
    if len <= 0 {
        return Err(cut_err("map len <= 0 is invalid"));
    }
    let mut res = RespMap::new();
    let count = len as usize / 2;
    for _ in 0..count {
        let key = preceded('+', parse_string).parse_next(input)?;
        let value = parse_frame(input)?;
        res.insert(key, value);
    }
    Ok(res)
}

fn parse_string(input: &mut &[u8]) -> PResult<String> {
    terminated(take_until(0.., CRLF), CRLF)
        .map(|v: &[u8]| String::from_utf8_lossy(v).into_owned())
        .parse_next(input)
}

pub(crate) fn cut_err(_s: impl Into<String>) -> ErrMode<ContextError> {
    ErrMode::Cut(ContextError::default())
}
