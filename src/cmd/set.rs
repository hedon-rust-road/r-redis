use std::collections::HashSet;

use crate::{BulkString, RespArray, RespFrame};

use super::{err::CommandError, extract_args, validate_command, CommandExecutor, SAdd, SIsMember};

impl CommandExecutor for SAdd {
    fn execute(self, backend: &crate::backend::Backend) -> crate::RespFrame {
        backend.sadd(self.key, self.member).into()
    }
}

impl CommandExecutor for SIsMember {
    fn execute(self, backend: &crate::backend::Backend) -> RespFrame {
        backend.is_member(self.key, self.member).into()
    }
}

impl TryFrom<RespArray> for SAdd {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, "sadd", value.len() - 1)?;
        let mut args = extract_args(value, 1)?.into_iter();
        let key = match args.next() {
            Some(RespFrame::BulkString(BulkString(Some(key)))) => {
                String::from_utf8(key).map_err(CommandError::Utf8Error)?
            }
            _ => {
                return Err(CommandError::InvalidArgument(
                    "Invalid arguments for sadd".into(),
                ))
            }
        };
        let mut member = HashSet::new();
        for mem in args {
            match mem {
                RespFrame::BulkString(mem) => {
                    member.insert(mem);
                }
                _ => {
                    return Err(CommandError::InvalidArgument(
                        "Invalid arguments for sadd".into(),
                    ));
                }
            }
        }
        Ok(SAdd { key, member })
    }
}

impl TryFrom<RespArray> for SIsMember {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, "sismember", 2)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (
                Some(RespFrame::BulkString(BulkString(Some(key)))),
                Some(RespFrame::BulkString(member)),
            ) => Ok(SIsMember {
                key: String::from_utf8(key).map_err(CommandError::Utf8Error)?,
                member,
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid arguments for sismember".into(),
            )),
        }
    }
}
