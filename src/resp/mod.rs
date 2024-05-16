use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

use enum_dispatch::enum_dispatch;

pub mod decode;
pub mod encode;

#[enum_dispatch]
pub trait RespEncode {
    fn encode(self) -> Vec<u8>;
}

pub trait RespDecode {
    fn decode(buf: Self) -> Result<RespFrame, String>;
}

/// RESP(Redis serialization protocol specification).
/// According to https://redis.io/docs/latest/develop/reference/protocol-spec/.
#[enum_dispatch(RespEncode)]
pub enum RespFrame {
    SimpleString(SimpleString),
    Error(SimpleError),

    NullBulkString(RespNullBulkString),
    NullArray(RespNullArray),
    Null(RespNull),

    Integer(i64),
    BulkString(BulkString),
    Array(RespArray),
    Boolean(bool),
    Double(f64),
    Map(RespMap),
    Set(RespSet),
}

pub struct SimpleString(String);
pub struct SimpleError(String);
pub struct RespNullBulkString;
pub struct RespNullArray;
pub struct RespNull;
pub struct RespArray(Vec<RespFrame>);
pub struct BulkString(Vec<u8>);
pub struct RespMap(HashMap<String, RespFrame>);
pub struct RespSet(HashSet<RespFrame>);

impl SimpleString {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleString(s.into())
    }
}

impl SimpleError {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleError(s.into())
    }
}

impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString(s.into())
    }
}

impl RespArray {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(s.into())
    }
}

impl RespMap {
    pub fn new() -> Self {
        RespMap(HashMap::new())
    }
}

impl RespSet {
    pub fn new() -> Self {
        RespSet(HashSet::new())
    }
}

impl Default for RespMap {
    fn default() -> Self {
        RespMap::new()
    }
}

impl Default for RespSet {
    fn default() -> Self {
        RespSet::new()
    }
}

impl Deref for BulkString {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for RespArray {
    type Target = Vec<RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for RespMap {
    type Target = HashMap<String, RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for RespSet {
    type Target = HashSet<RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
