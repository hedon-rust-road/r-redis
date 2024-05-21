use crate::{RespArray, RespFrame};

use super::{err::CommandError, extract_args, validate_command, CommandExecutor, SAdd};

impl CommandExecutor for SAdd {
    fn execute(self, backend: &crate::backend::Backend) -> crate::RespFrame {
        backend.sadd(self.key, self.field).into()
    }
}

impl TryFrom<RespArray> for SAdd {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, "sadd", 2)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(field))) => Ok(SAdd {
                key: String::from_utf8(key.1).map_err(CommandError::Utf8Error)?,
                field: String::from_utf8(field.1).map_err(CommandError::Utf8Error)?,
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid arguments for sadd".into(),
            )),
        }
    }
}
