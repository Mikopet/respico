use crate::{error::*, value::*};

const CRLF: &str = "\r\n";

/// Main parser function
pub fn parse(s: &str) -> Result<Value, Error> {
    match &s.split_once(CRLF) {
        // If there were no valid line break, we try to parse simple expression
        None => Value::init(&s),
        // if there is valid line break, we start recursive processing
        Some((line, rest)) => parse_recursive(&line, &rest),
    }
}

fn parse_recursive<'a>(line: &'a str, rest: &'a str) -> Result<Value<'a>, Error> {
    let mut result = Value::init(&line)?;

    // short circuit if no more lines to parse (simple types)
    if !&rest.is_empty() {
        let (mut current, mut rest) = &rest.split_once(CRLF).unwrap();
        // check function root type
        result = match result {
            // short circuit for null Array and BulkString
            null if null.is_null() => null,

            // if it is BulkString, save current data line
            Value::BulkString(mut bs) => {
                bs.extend(current.as_bytes());
                Value::BulkString(bs)
            }
            // if it is Array, call recursion and save after
            Value::Array(mut a) => {
                loop {
                    let r = parse_recursive(&current, &rest)?;
                    let steps = count_steps(&r);
                    a.push(r);

                    // break if this subset is already full
                    if a.len() == a.capacity() {
                        break;
                    }

                    for _ in 0..steps {
                        match rest.split_once(CRLF) {
                            None => break,
                            Some((c, r)) => {
                                current = c;
                                rest = r;
                            }
                        }
                    }
                }

                Value::Array(a)
            }
            r => r,
        };
    }

    Ok(result)
}

fn count_steps(r: &Value) -> usize {
    // println!("{:?}", r);
    let count = match r {
        Value::Array(a) => a.iter().map(|v| count_steps(v)).sum::<usize>() + 1,
        Value::BulkString(_) => 2,
        _ => 1,
    };

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    #[test]
    fn parse_valid_bulk() {
        let result = parse("$2\r\n22\r\n").unwrap();
        assert_eq!(result, Value::BulkString(vec![b'2', b'2']));
    }

    #[test]
    fn parse_valid_bulk_string() {
        let map = HashMap::from([
            ("$0\r\n\r\n", Value::BulkString(vec![])),
            ("$1\r\n1\r\n", Value::BulkString(vec![b'1'])),
        ]);

        for (k, v) in map {
            assert_eq!(parse(k).unwrap(), v);
        }
    }

    #[test]
    fn parse_valid_array() {
        let map = HashMap::from([
            ("*0\r\n", Value::Array(vec![])),
            ("*1\r\n:1\r\n", Value::Array(vec![Value::Integer(1)])),
            (
                "*2\r\n:1\r\n:2\r\n",
                Value::Array(vec![Value::Integer(1), Value::Integer(2)]),
            ),
            (
                "*3\r\n:1\r\n:2\r\n$1\r\n0\r\n",
                Value::Array(vec![
                    Value::Integer(1),
                    Value::Integer(2),
                    Value::BulkString(vec![b'0']),
                ]),
            ),
            (
                "*3\r\n$1\r\n1\r\n:2\r\n$1\r\n0\r\n",
                Value::Array(vec![
                    Value::BulkString(vec![b'1']),
                    Value::Integer(2),
                    Value::BulkString(vec![b'0']),
                ]),
            ),
        ]);

        for (k, v) in map {
            assert_eq!(parse(k).unwrap(), v);
        }
    }

    #[test]
    fn parse_valid_nested_array() {
        let map = HashMap::from([
            (
                "*1\r\n*1\r\n:1\r\n",
                Value::Array(vec![Value::Array(vec![Value::Integer(1)])]),
            ),
            (
                "*1\r\n*2\r\n:1\r\n:2\r\n",
                Value::Array(vec![Value::Array(vec![
                    Value::Integer(1),
                    Value::Integer(2),
                ])]),
            ),
            (
                "*2\r\n*2\r\n:1\r\n:2\r\n$5\r\nhi\r\n",
                Value::Array(vec![
                    Value::Array(vec![Value::Integer(1), Value::Integer(2)]),
                    Value::BulkString(vec![b'h', b'i']),
                ]),
            ),
            (
                "*5\r\n*1\r\n:1\r\n:2\r\n$2\r\nhi\r\n*0\r\n*1\r\n*2\r\n$0\r\n\r\n$1\r\n \r\n",
                Value::Array(vec![
                    Value::Array(vec![Value::Integer(1)]),
                    Value::Integer(2),
                    Value::BulkString(vec![b'h', b'i']),
                    Value::Array(vec![]),
                    Value::Array(vec![Value::Array(vec![
                        Value::BulkString(vec![]),
                        Value::BulkString(vec![b' ']),
                    ])]),
                ]),
            ),
        ]);

        for (k, v) in map {
            assert_eq!(parse(k).unwrap(), v);
        }
    }
}
