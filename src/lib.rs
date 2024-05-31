mod types;

use types::*;

fn parse_type(s: &str) -> Option<Value> {
    Value::init(s.trim()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

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
            assert_eq!(parse_type(k).unwrap(), v);
        }
    }

    #[test]
    fn parse_invalid_types() {
        let result = parse_type("invalid");
        assert_eq!(result, None);
    }
}
