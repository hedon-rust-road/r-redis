pub mod echo;
pub mod err;
pub mod hmap;
pub mod map;

use enum_dispatch::enum_dispatch;

use crate::{backend, RespArray, RespFrame, SimpleString};

use self::err::CommandError;

lazy_static::lazy_static! {
    static ref RESP_OK:RespFrame = SimpleString::new("OK").into();
}

#[enum_dispatch]
pub trait CommandExecutor {
    fn execute(self, backend: &backend::Backend) -> RespFrame;
}

#[enum_dispatch(CommandExecutor)]
pub enum Command {
    Get(Get),
    Set(Set),
    HGet(HGet),
    HSet(HSet),
    HGetAll(HGetAll),
    HMGet(HMGet),
    Echo(Echo),
}

#[derive(Debug)]
pub struct Get {
    key: String,
}

#[derive(Debug)]
pub struct Set {
    key: String,
    value: RespFrame,
}

#[derive(Debug)]
pub struct HGet {
    key: String,
    field: String,
}

#[derive(Debug)]
pub struct HSet {
    key: String,
    field: String,
    value: RespFrame,
}

#[derive(Debug)]
pub struct HGetAll {
    key: String,
}

#[derive(Debug)]
pub struct HMGet {
    key: String,
    fields: Vec<String>,
}

#[derive(Debug)]
pub struct Echo {
    message: String,
}

impl TryFrom<RespFrame> for Command {
    type Error = CommandError;

    fn try_from(value: RespFrame) -> Result<Self, Self::Error> {
        match value {
            RespFrame::Array(v) => v.try_into(),
            _ => Err(CommandError::InvalidCommand(
                "Command must be an Array".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for Command {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        match value.first() {
            Some(RespFrame::BulkString(ref c)) => match c.as_ref() {
                b"get" => Ok(Get::try_from(value)?.into()),
                b"set" => Ok(Set::try_from(value)?.into()),
                b"hget" => Ok(HGet::try_from(value)?.into()),
                b"hset" => Ok(HSet::try_from(value)?.into()),
                b"hgetall" => Ok(HGetAll::try_from(value)?.into()),
                b"hmget" => Ok(HMGet::try_from(value)?.into()),
                b"echo" => Ok(Echo::try_from(value)?.into()),
                _ => Err(CommandError::InvalidCommand(format!(
                    "Invalid command: {}",
                    String::from_utf8_lossy(c.as_ref())
                ))),
            },
            _ => Err(CommandError::InvalidCommand(
                "Command must have a BulkString as the first argument".to_string(),
            )),
        }
    }
}

fn validate_command(
    value: &RespArray,
    cmd: &str,
    n_arg: usize,
) -> anyhow::Result<(), CommandError> {
    if value.len() != n_arg + 1 {
        return Err(CommandError::InvalidArgument(format!(
            "length of {} command arguments must be {}",
            cmd, n_arg
        )));
    }

    match value[0] {
        RespFrame::BulkString(ref c) => {
            if c.to_ascii_lowercase() != cmd.as_bytes() {
                return Err(CommandError::InvalidArgument(format!(
                    "Invalid command: expected: {}, got: {}",
                    cmd,
                    String::from_utf8_lossy(c)
                )));
            }
        }
        _ => {
            return Err(CommandError::InvalidCommand(
                "Command must have a BulkString as the first argument".to_string(),
            ))
        }
    }

    Ok(())
}

fn extract_args(value: RespArray, start: usize) -> anyhow::Result<Vec<RespFrame>, CommandError> {
    Ok(value.1.into_iter().skip(start).collect::<Vec<RespFrame>>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_command() -> anyhow::Result<()> {
        // valid command
        let value = RespArray::new(vec![
            RespFrame::BulkString("get".into()),
            RespFrame::BulkString("key".into()),
        ]);
        validate_command(&value, "get", 1)?;

        // invalid command
        let value = RespArray::new(vec![
            RespFrame::BulkString("get".into()),
            RespFrame::BulkString("key".into()),
            RespFrame::BulkString("key".into()),
        ]);
        let res = validate_command(&value, "get", 1);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Invalid argument: length of get command arguments must be 1".to_string()
        );
        Ok(())
    }
}
