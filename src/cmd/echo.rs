use crate::{BulkString, RespArray, RespFrame, SimpleString};

use super::{err::CommandError, extract_args, validate_command, CommandExecutor, Echo};

impl CommandExecutor for Echo {
    fn execute(self, _backend: &crate::backend::Backend) -> crate::RespFrame {
        SimpleString::new(self.message).into()
    }
}

impl TryFrom<RespArray> for Echo {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, "echo", 1)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(BulkString(Some(message)))) => Ok(Echo {
                message: String::from_utf8(message).map_err(CommandError::Utf8Error)?,
            }),
            _ => Err(CommandError::InvalidArgument(
                "Echo command requires a single bulk string argument".to_string(),
            )),
        }
    }
}
