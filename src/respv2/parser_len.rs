use std::num::NonZeroUsize;

use winnow::{
    combinator::{dispatch, fail, terminated},
    error::{ErrMode, Needed},
    token::{any, take_until},
    PResult, Parser,
};

use crate::{
    err::RespError,
    respv2::parser::{cut_err, integer},
    CRLF,
};

pub fn parse_frame_length(input: &[u8]) -> Result<usize, RespError> {
    let target = &mut (&*input);
    let ret = parse_frame_len(target);
    match ret {
        Ok(_) => {
            let start = input.as_ptr() as usize;
            let end = target.as_ptr() as usize;
            Ok(end - start)
        }
        Err(_) => Err(RespError::NotCompleted),
    }
}

pub fn parse_frame_len(input: &mut &[u8]) -> PResult<()> {
    // parse simple frame like {}...\r\n
    let mut simple_parser = terminated(take_until(0.., CRLF), CRLF).value(());
    dispatch!(any;
        b'+' => simple_parser,
        b'-' => simple_parser,
        b':' => simple_parser,
        b'$' => bulk_string_len,
        b'*' => array_len,
        b'_' => simple_parser,
        b'#' => simple_parser,
        b',' => simple_parser,
        b'%' => map_len,
        _v => fail::<_,_,_>
    )
    .parse_next(input)
}

fn array_len(input: &mut &[u8]) -> PResult<()> {
    let len: i64 = integer.parse_next(input)?;
    if len == 0 || len == -1 {
        return Ok(());
    } else if len < -1 {
        return Err(cut_err("array length must >= -1"));
    }
    for _ in 0..len {
        parse_frame_len(input)?;
    }
    Ok(())
}

fn bulk_string_len(input: &mut &[u8]) -> PResult<()> {
    let len = integer.parse_next(input)?;
    if len == -1 || len == 0 {
        return Ok(());
    } else if len < -1 {
        return Err(cut_err("bulk string length must >= -1"));
    }
    // terminated(take(len as usize), CRLF)
    //     .value(())
    //     .parse_next(input)

    // just skip the data and do not parse it.
    // because we just need the length of the data.
    let len_with_crlf = len as usize + 2;
    if input.len() < len_with_crlf {
        let size = NonZeroUsize::new((len_with_crlf - input.len()) as usize).unwrap();
        return Err(ErrMode::Incomplete(Needed::Size(size)));
    }
    *input = &input[(len + 2) as usize..];
    Ok(())
}

fn map_len(input: &mut &[u8]) -> PResult<()> {
    let len = integer.parse_next(input)?;
    if len <= 0 {
        return Err(cut_err("map length must > 0"));
    }
    let count = len as usize / 2;
    for _ in 0..count {
        // key
        terminated(take_until(0.., CRLF), CRLF)
            .value(())
            .parse_next(input)?;
        // value
        parse_frame_len(input)?;
    }
    Ok(())
}
