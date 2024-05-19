use crate::{CommandError, CommandExecutor, HGet, HGetAll, HSet, RespArray, RespFrame};

use super::{extract_args, validate_command};

impl CommandExecutor for HGet {
    fn execute(self) -> crate::RespFrame {
        todo!()
    }
}

impl CommandExecutor for HSet {
    fn execute(self) -> crate::RespFrame {
        todo!()
    }
}

impl CommandExecutor for HGetAll {
    fn execute(self) -> crate::RespFrame {
        todo!()
    }
}

impl TryFrom<RespArray> for HGet {
    type Error = CommandError;

    // hget key field
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, "hget", 2)?;

        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(field))) => Ok(HGet {
                key: String::from_utf8(key.0).map_err(CommandError::Utf8Error)?,
                field: String::from_utf8(field.0).map_err(CommandError::Utf8Error)?,
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid key or field".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for HSet {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, "hset", 3)?;

        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(field)), Some(value)) => {
                Ok(HSet {
                    key: String::from_utf8(key.0).map_err(CommandError::Utf8Error)?,
                    field: String::from_utf8(field.0).map_err(CommandError::Utf8Error)?,
                    value,
                })
            }
            _ => Err(CommandError::InvalidArgument(
                "Invalid key, field or value".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for HGetAll {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, "hgetall", 1)?;

        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(key)) => Ok(HGetAll {
                key: String::from_utf8(key.0).map_err(CommandError::Utf8Error)?,
            }),
            _ => Err(CommandError::InvalidArgument("Invalid key".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::BulkString;

    use super::*;

    #[test]
    fn test_hget_from_resp_array() -> anyhow::Result<()> {
        let resp_array = RespArray::new(vec![
            BulkString::new("hget").into(),
            BulkString::new("key").into(),
            BulkString::new("field").into(),
        ]);
        let hget = HGet::try_from(resp_array)?;
        assert_eq!(hget.key, "key");
        assert_eq!(hget.field, "field");

        Ok(())
    }

    #[test]
    fn test_hset_from_resp_array() -> anyhow::Result<()> {
        let resp_array = RespArray::new(vec![
            BulkString::new("hset").into(),
            BulkString::new("key").into(),
            BulkString::new("field").into(),
            BulkString::new("value").into(),
        ]);
        let hget = HSet::try_from(resp_array)?;
        assert_eq!(hget.key, "key");
        assert_eq!(hget.field, "field");
        assert_eq!(hget.value, RespFrame::BulkString(BulkString::new("value")));

        Ok(())
    }

    #[test]
    fn test_hgetall_from_resp_array() -> anyhow::Result<()> {
        let resp_array = RespArray::new(vec![
            BulkString::new("hgetall").into(),
            BulkString::new("key").into(),
        ]);
        let hget = HGetAll::try_from(resp_array)?;
        assert_eq!(hget.key, "key");
        Ok(())
    }
}
