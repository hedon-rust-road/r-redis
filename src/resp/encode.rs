use crate::{
    BulkString, RespArray, RespEncode, RespMap, RespNull, RespNullArray, RespNullBulkString,
    RespSet, SimpleError, SimpleString,
};

const BUF_CAP: usize = 4096;

/// Simple strings are encoded as a plus (+) character, followed by a string.
/// The string mustn't contain a CR (\r) or LF (\n) character and is terminated by CRLF (i.e., \r\n).
///
/// Examples: +OK\r\n
impl RespEncode for SimpleString {
    fn encode(self) -> Vec<u8> {
        format!("+{}\r\n", self.0).into_bytes()
    }
}

/// Simple errors, or simply just errors, are similar to simple strings,
/// but their first character is the minus (-) character.
///
/// The difference between simple strings and errors in RESP is
/// that clients should treat errors as exceptions,
/// whereas the string encoded in the error type is the error message itself.
///
/// Examples: -Error message\r\n
impl RespEncode for SimpleError {
    fn encode(self) -> Vec<u8> {
        format!("-{}\r\n", self.0).into_bytes()
    }
}

/// The null bulk string represents a non-existing value.
/// The `GET`` command returns the Null Bulk String when the target key doesn't exist.
///
/// Examples: $-1\r\n
impl RespEncode for RespNullBulkString {
    fn encode(self) -> Vec<u8> {
        b"$-1\r\n".to_vec()
    }
}

/// Null arrays exist as an alternative way of representing a null value.
/// For instance, when the BLPOP command times out, it returns a null array.
///
/// Examples: *-1\r\n
impl RespEncode for RespNullArray {
    fn encode(self) -> Vec<u8> {
        b"*-1\r\n".to_vec()
    }
}

/// The null data type represents non-existent values.
///
/// Examples: _\r\n
impl RespEncode for RespNull {
    fn encode(self) -> Vec<u8> {
        b"_\r\n".to_vec()
    }
}

/// This type is a CRLF-terminated string that represents a signed, base-10, 64-bit integer.
///
/// Format:
///     :[<+|->]<value>\r\n
///
/// - The colon (:) as the first byte.
/// - An optional plus (+) or minus (-) as the sign.
/// - One or more decimal digits (0..9) as the integer's unsigned, base-10 value.
/// - The CRLF terminator.
impl RespEncode for i64 {
    fn encode(self) -> Vec<u8> {
        format!(":{}\r\n", self).into_bytes()
    }
}

/// A bulk string represents a single binary string.
/// The string can be of any size, but by default,
/// Redis limits it to 512 MB (see the proto-max-bulk-len configuration directive).
///
/// Format:
///     $<length>\r\n<data>\r\n
///
/// - The dollar sign ($) as the first byte.
/// - One or more decimal digits (0..9) as the string's length, in bytes, as an unsigned, base-10 value.
/// - The CRLF terminator.
/// - The data.
/// - A final CRLF.
impl RespEncode for BulkString {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.len() + 16);
        buf.extend_from_slice(&format!("${}\r\n", self.len()).into_bytes());
        buf.extend_from_slice(&self);
        buf.extend_from_slice(b"\r\n");
        buf
    }
}

/// Clients send commands to the Redis server as RESP arrays.
/// Similarly, some Redis commands that return collections of
/// elements use arrays as their replies.
///
/// Format:
///     *<number-of-elements>\r\n<element-1>...<element-n>
///
/// - An asterisk (*) as the first byte.
/// - One or more decimal digits (0..9) as the number of elements in the array as an unsigned, base-10 value.
/// - The CRLF terminator.
/// - An additional RESP type for every element of the array.
impl RespEncode for RespArray {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(format!("*{}\r\n", self.len()).as_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.encode())
        }
        buf
    }
}

/// #<t|f>\r\n
impl RespEncode for bool {
    fn encode(self) -> Vec<u8> {
        format!("#{}\r\n", if self { "t" } else { "f" }).into_bytes()
    }
}

/// The Double RESP type encodes a double-precision floating point value.
/// Format:
///     ,[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n
///
/// - The comma character (,) as the first byte.
/// - An optional plus (+) or minus (-) as the sign.
/// - One or more decimal digits (0..9) as an unsigned, base-10 integral value.
/// - An optional dot (.), followed by one or more decimal digits (0..9) as an unsigned, base-10 fractional value.
/// - An optional capital or lowercase letter E (E or e),
///     followed by an optional plus (+) or minus (-) as the exponent's sign,
///     ending with one or more decimal digits (0..9) as an unsigned, base-10 exponent value.
/// - The CRLF terminator.
///
/// Example:
///     1.23
///     ,1.23\r\n
///
/// Other examples:
///     ,inf\r\n
///     ,-inf\r\n
///     ,nan\r\n
impl RespEncode for f64 {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(32);
        let ret = if self.abs() > 1e+8 || self.abs() < 1e-8 {
            format!(",{:e}\r\n", self)
        } else {
            let sign = if self < 0.0 || self.is_nan() { "" } else { "+" };
            format!(",{}{}\r\n", sign, self)
        };
        let ret = ret.to_lowercase();
        buf.extend_from_slice(&ret.into_bytes());
        buf
    }
}

/// The RESP map encodes a collection of key-value tuples, i.e., a dictionary or a hash.
/// Format:
///     %<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>
///
/// - A percent character (%) as the first byte.
/// - One or more decimal digits (0..9) as the number of entries, or key-value tuples, in the map as an unsigned, base-10 value.
/// - The CRLF terminator.
/// - Two additional RESP types for every key and value in the map.
///
/// Examples:
///     {
///         "first": 1,
///         "second": 2
///     }
///            ↓
///         %2\r\n
///         +first\r\n
///         :1\r\n
///         +second\r\n
///         :2\r\n
/// (The raw RESP encoding is split into multiple lines for readability).
impl RespEncode for RespMap {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("%{}\r\n", self.len()).into_bytes());
        for (key, value) in self.0 {
            buf.extend(SimpleString::new(key).encode());
            buf.extend(&value.encode());
        }
        println!("{}", String::from_utf8(buf.clone()).unwrap());
        buf
    }
}

/// Sets are somewhat like Arrays but are unordered and should only contain unique elements.
/// Format:
///     ~<number-of-elements>\r\n<element-1>...<element-n>
///
/// - A tilde (~) as the first byte.
/// - One or more decimal digits (0..9) as the number of elements in the set as an unsigned, base-10 value.
/// - The CRLF terminator.
/// - An additional RESP type for every element of the Set.
impl RespEncode for RespSet {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("~{}\r\n", self.len()).into_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.encode());
        }
        buf
    }
}

#[cfg(test)]
mod tests {
    use std::f64::{INFINITY, NAN};

    use crate::RespFrame;

    use super::*;

    #[test]
    fn test_simple_string_encode() {
        let frame: RespFrame = SimpleString::new("OK".to_string()).into();
        assert_eq!(frame.encode(), b"+OK\r\n");
        let frame: RespFrame = SimpleString::new("hello".to_string()).into();
        assert_eq!(frame.encode(), b"+hello\r\n");
    }

    #[test]
    fn test_simple_error_encode() {
        let frame: RespFrame = SimpleError::new("Error Message".to_string()).into();
        assert_eq!(frame.encode(), b"-Error Message\r\n");
    }

    #[test]
    fn test_null_bulk_string_encode() {
        let frame: RespFrame = RespNullBulkString.into();
        assert_eq!(frame.encode(), b"$-1\r\n");
    }

    #[test]
    fn test_null_array_encode() {
        let frame: RespFrame = RespNullArray.into();
        assert_eq!(frame.encode(), b"*-1\r\n");
    }

    #[test]
    fn test_null_encode() {
        let frame: RespFrame = RespNull.into();
        assert_eq!(frame.encode(), b"_\r\n");
    }

    #[test]
    fn test_integer_encode() {
        let frame: RespFrame = 0.into();
        assert_eq!(frame.encode(), b":0\r\n");
        let frame: RespFrame = (-123).into();
        assert_eq!(frame.encode(), b":-123\r\n");
        let frame: RespFrame = (123).into();
        assert_eq!(frame.encode(), b":123\r\n");
    }

    #[test]
    fn test_bulk_string_encode() {
        let frame: RespFrame = BulkString::new(b"hello").into();
        assert_eq!(frame.encode(), b"$5\r\nhello\r\n");
    }

    #[test]
    fn test_array_encode() {
        let frame: RespFrame = RespArray::new(vec![
            SimpleString::new("hello").into(),
            SimpleError::new("Err").into(),
            123.into(),
        ])
        .into();
        assert_eq!(frame.encode(), b"*3\r\n+hello\r\n-Err\r\n:123\r\n");
    }

    #[test]
    fn test_boolean_encode() {
        let frame: RespFrame = true.into();
        assert_eq!(frame.encode(), b"#t\r\n");
        let frame: RespFrame = false.into();
        assert_eq!(frame.encode(), b"#f\r\n");
    }

    #[test]
    fn test_double_encode() {
        let frame: RespFrame = (1.22).into();
        assert_eq!(frame.encode(), b",+1.22\r\n");
        let frame: RespFrame = (-1.22).into();
        assert_eq!(frame.encode(), b",-1.22\r\n");
        let frame: RespFrame = (0.0).into();
        assert_eq!(frame.encode(), b",0e0\r\n");
        let frame: RespFrame = (0.00000).into();
        assert_eq!(frame.encode(), b",0e0\r\n");
        let frame: RespFrame = (INFINITY).into();
        assert_eq!(frame.encode(), b",inf\r\n");
        let frame: RespFrame = (-INFINITY).into();
        assert_eq!(frame.encode(), b",-inf\r\n");
        let frame: RespFrame = (NAN).into();
        assert_eq!(frame.encode(), b",nan\r\n");
    }

    #[test]
    fn test_map_encode() {
        let mut map = RespMap::new();
        map.insert("first".to_string(), 1.into());
        map.insert("second".to_string(), 2.into());
        let frame: RespFrame = map.into();
        assert_eq!(frame.encode(), b"%2\r\n+first\r\n:1\r\n+second\r\n:2\r\n");
    }

    #[test]
    fn test_set_encode() {
        let set = RespSet::new(vec![1.into(), 2.into()]);
        let frame: RespFrame = set.into();
        assert_eq!(frame.encode(), b"~2\r\n:1\r\n:2\r\n");
    }
}
