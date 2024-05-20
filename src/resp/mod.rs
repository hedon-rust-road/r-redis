use bytes::BytesMut;
use enum_dispatch::enum_dispatch;

use self::err::RespError;

pub use self::{
    array::RespArray, bulk_string::BulkString, map::RespMap, null::RespNull, resp_frame::RespFrame,
    set::RespSet, simple_error::SimpleError, simple_string::SimpleString,
};

pub mod array;
pub mod boolean;
pub mod bulk_string;
pub mod decode;
pub mod double;
pub mod encode;
pub mod err;
pub mod integer;
pub mod map;
pub mod null;
pub mod resp_frame;
pub mod set;
pub mod simple_error;
pub mod simple_string;

#[enum_dispatch]
pub trait RespEncode {
    fn encode(self) -> Vec<u8>;
}

pub trait RespDecode: Sized {
    const PREFIX: &'static str;
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError>;
    fn expect_length(buf: &[u8]) -> Result<usize, RespError>;
}
