use crate::{Backend, BulkString, RespArray, RespFrame, RespMap, RespNull};

use super::{
    extract_args, validate_command, CommandError, CommandExecutor, HGet, HGetAll, HMGet, HSet,
    RESP_OK,
};

impl CommandExecutor for HGet {
    fn execute(self, backend: &Backend) -> crate::RespFrame {
        let res = backend.hget(&self.key, &self.field);
        match res {
            Some(value) => value,
            None => RespFrame::Null(RespNull),
        }
    }
}

impl CommandExecutor for HSet {
    fn execute(self, backend: &Backend) -> crate::RespFrame {
        backend.hset(self.key, self.field, self.value);
        RESP_OK.clone()
    }
}

impl CommandExecutor for HGetAll {
    fn execute(self, backend: &Backend) -> crate::RespFrame {
        let res = backend.hgetall(&self.key);
        let mut m = RespMap::new();
        if let Some(map) = res {
            for (k, v) in map {
                m.insert(k, v);
            }
        }
        m.into()
    }
}

impl CommandExecutor for HMGet {
    fn execute(self, backend: &crate::backend::Backend) -> RespFrame {
        let m = backend.hmget(&self.key, &self.fields);
        let mut res = RespMap::new();
        for (k, v) in m {
            res.insert(k, v);
        }
        res.into()
    }
}

impl TryFrom<RespArray> for HGet {
    type Error = CommandError;

    // hget key field
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, "hget", 2)?;

        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (
                Some(RespFrame::BulkString(BulkString(Some(key)))),
                Some(RespFrame::BulkString(BulkString(Some(field)))),
            ) => Ok(HGet {
                key: String::from_utf8(key).map_err(CommandError::Utf8Error)?,
                field: String::from_utf8(field).map_err(CommandError::Utf8Error)?,
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
            (
                Some(RespFrame::BulkString(BulkString(Some(key)))),
                Some(RespFrame::BulkString(BulkString(Some(field)))),
                Some(value),
            ) => Ok(HSet {
                key: String::from_utf8(key).map_err(CommandError::Utf8Error)?,
                field: String::from_utf8(field).map_err(CommandError::Utf8Error)?,
                value,
            }),
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
            Some(RespFrame::BulkString(BulkString(Some(key)))) => Ok(HGetAll {
                key: String::from_utf8(key).map_err(CommandError::Utf8Error)?,
            }),
            _ => Err(CommandError::InvalidArgument("Invalid key".to_string())),
        }
    }
}

impl TryFrom<RespArray> for HMGet {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        if value.len() < 3 {
            return Err(CommandError::InvalidArgument(
                "wrong number of arguments for 'hmget' command".to_string(),
            ));
        }

        validate_command(&value, "hmget", value.len() - 1)?;

        let mut args = extract_args(value, 1)?.into_iter();
        let key = match args.next() {
            Some(RespFrame::BulkString(BulkString(Some(key)))) => {
                String::from_utf8(key).map_err(CommandError::Utf8Error)?
            }
            _ => {
                return Err(CommandError::InvalidArgument(
                    "Invalid of lack of key".to_string(),
                ))
            }
        };

        let mut res = HMGet {
            key,
            fields: Vec::new(),
        };

        for arg in args {
            match arg {
                RespFrame::BulkString(BulkString(Some(field))) => res
                    .fields
                    .push(String::from_utf8(field).map_err(CommandError::Utf8Error)?),
                _ => {
                    return Err(CommandError::InvalidArgument(
                        "Invalid of lack of field".to_string(),
                    ))
                }
            }
        }
        Ok(res)
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
