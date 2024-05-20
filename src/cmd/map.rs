use crate::{Backend, RespArray, RespFrame, RespNull};

use super::{extract_args, validate_command, CommandError, CommandExecutor, Get, Set, RESP_OK};

impl CommandExecutor for Get {
    fn execute(self, backend: &Backend) -> RespFrame {
        let res = backend.get(&self.key);
        match res {
            Some(value) => value,
            None => RespFrame::Null(RespNull),
        }
    }
}

impl CommandExecutor for Set {
    fn execute(self, backend: &Backend) -> RespFrame {
        backend.set(self.key, self.value);
        RESP_OK.clone()
    }
}

impl TryFrom<RespArray> for Get {
    type Error = CommandError;

    // get key
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, "get", 1)?;

        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(key)) => Ok(Get {
                key: String::from_utf8(key.to_vec())
                    .map_err(|e| CommandError::InvalidArgument(format!("invalid utf8: {}", e)))?,
            }),
            _ => Err(CommandError::InvalidArgument("Invalid key".to_string())),
        }
    }
}

impl TryFrom<RespArray> for Set {
    type Error = CommandError;

    // set key value
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, "set", 2)?;

        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(value))) => Ok(Set {
                key: String::from_utf8(key.0).map_err(CommandError::Utf8Error)?,
                value: value.into(),
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid key or value".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use super::*;
    use crate::{RespArray, RespDecode};

    #[test]
    fn test_get_from_resp_array() -> anyhow::Result<()> {
        // test from RespArray
        let resp_array = RespArray::new(vec![
            RespFrame::BulkString("get".into()),
            RespFrame::BulkString("key".into()),
        ]);
        let get = Get::try_from(resp_array)?;
        assert_eq!(get.key, "key");

        // test from bytes
        let mut buf = BytesMut::from("*2\r\n$3\r\nget\r\n$3\r\nkey\r\n");
        let resp_array = RespArray::decode(&mut buf)?;
        let get = Get::try_from(resp_array)?;
        assert_eq!(get.key, "key");

        // invalid command
        let mut buf = BytesMut::from("*2\r\n$4\r\nxget\r\n$3\r\nkey\r\n");
        let resp_array = RespArray::decode(&mut buf)?;
        let result = Get::try_from(resp_array);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid argument: Invalid command: expected: get, got: xget",
        );

        // invalid argument
        let mut buf = BytesMut::from("*3\r\n$3\r\nget\r\n$3\r\nkey\r\n$4\r\nkey2\r\n");
        let resp_array = RespArray::decode(&mut buf)?;
        let result = Get::try_from(resp_array);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid argument: length of get command arguments must be 1",
        );
        Ok(())
    }

    #[test]
    fn test_set_from_resp_array() -> anyhow::Result<()> {
        // valid case
        let resp_array = RespArray::new(vec![
            RespFrame::BulkString("set".into()),
            RespFrame::BulkString("key".into()),
            RespFrame::BulkString("value".into()),
        ]);
        let result = Set::try_from(resp_array)?;
        assert_eq!(result.key, "key".to_string());
        assert_eq!(result.value, RespFrame::BulkString("value".into()));

        // invalid case - cmd error
        let resp_array = RespArray::new(vec![
            RespFrame::BulkString("setx".into()),
            RespFrame::BulkString("key".into()),
            RespFrame::BulkString("value".into()),
        ]);

        let result = Set::try_from(resp_array);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid argument: Invalid command: expected: set, got: setx"
        );

        // invalid case - invalid argument error
        let resp_array = RespArray::new(vec![
            RespFrame::BulkString("set".into()),
            RespFrame::BulkString("key".into()),
            RespFrame::BulkString("value".into()),
            RespFrame::BulkString("value2".into()),
        ]);
        let result = Set::try_from(resp_array);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid argument: length of set command arguments must be 2".to_string()
        );
        Ok(())
    }

    #[test]
    fn test_execute_get() -> anyhow::Result<()> {
        Ok(())
    }
}
