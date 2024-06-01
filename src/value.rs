use crate::error::*;

#[derive(Debug, Eq, PartialEq)]
pub enum Value<'a> {
    // RESP2
    SimpleString(&'a str),
    SimpleError(&'a str),
    Integer(i64),
    BulkString(Vec<u8>),
    Array(Vec<Value<'a>>),
    // RESP3
    // Null
    // Bool,
    // Double,
    // BigNumber,
    // BulkError,
    // VerbatimString,
    // Map,
    // Set,
    // Push,
}

impl<'a> Value<'a> {
    /// Parsing RESP type string (without whitespace)
    ///
    /// "+OK" -> Value::SimpleString(String::from("OK")
    /// "$5"  -> Value::BulkString(String::with_capacity(5))
    ///
    pub fn init(s: &'a str) -> Result<Self, Error> {
        let first = s[..1].chars().next().unwrap();
        let rest = &s[1..];

        match first {
            '+' => Ok(Value::SimpleString(rest)),
            '-' => Ok(Value::SimpleError(rest)),
            ':' => {
                if let Ok(n) = rest.parse::<i64>() {
                    Ok(Value::Integer(n))
                } else {
                    Err(Error("invalid number"))
                }
            }
            '$' | '*' => {
                if let Ok(n) = rest.parse::<usize>() {
                    match first {
                        '$' => Ok(Value::BulkString(Vec::with_capacity(n))),
                        '*' => Ok(Value::Array(Vec::with_capacity(n))),
                        _ => unreachable!(),
                    }
                } else {
                    Err(Error("invalid length"))
                }
            }
            _ => Err(Error("invalid first char")),
        }
    }

    /// Checks if aggregate type is null (capacity == 0)
    ///
    /// If type is simple, giving fallback (false)
    pub fn is_null(&self) -> bool {
        let capacity = match self {
            Value::Array(a) => a.capacity(),
            Value::BulkString(s) => s.capacity(),
            // defaults for simple types
            _ => 1, // false
        };

        capacity == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    #[test]
    fn init_invalid_types() {
        let map = HashMap::from([
            ("data", Error("invalid first char")),
            ("$-2", Error("invalid length")),
            ("*-2", Error("invalid length")),
            (":i", Error("invalid number")),
        ]);

        for (k, v) in map {
            assert_eq!(Value::init(k), Err(v));
        }
    }

    #[test]
    fn parse_valid_types() {
        let map = HashMap::from([
            ("+OK", Value::SimpleString("OK")),
            ("+OK ok", Value::SimpleString("OK ok")),
            ("-ERR", Value::SimpleError("ERR")),
            ("-ERR reason", Value::SimpleError("ERR reason")),
            (":+2", Value::Integer(2)),
            (":1", Value::Integer(1)),
            (":0", Value::Integer(0)),
            (":-1", Value::Integer(-1)),
            ("$0", Value::BulkString(vec![])),
            ("$1", Value::BulkString(vec![])),
            ("*0", Value::Array(vec![])),
            ("*1", Value::Array(vec![])),
        ]);

        for (k, v) in map {
            assert_eq!(Value::init(k).unwrap(), v);
        }
    }
}
