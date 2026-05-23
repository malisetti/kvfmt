use crate::error::{KvfmtError, Result};
use serde_json::{Map, Value};

pub fn parse(input: &str) -> Result<Map<String, Value>> {
    let bytes = input.as_bytes();
    let mut map = Map::new();
    let mut pos = 0usize;
    while pos < bytes.len() {
        while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= bytes.len() {
            break;
        }
        let key_start = pos;
        while pos < bytes.len() && !bytes[pos].is_ascii_whitespace() && bytes[pos] != b'=' {
            pos += 1;
        }
        if pos == key_start {
            return parse_err(pos, "empty key");
        }
        let key = std::str::from_utf8(&bytes[key_start..pos])
            .map_err(|_| KvfmtError::LogfmtParse {
                pos: key_start,
                message: "invalid UTF-8 in key".into(),
            })?
            .to_string();

        if pos >= bytes.len() || bytes[pos] != b'=' {
            map.insert(key, Value::Bool(true));
            continue;
        }
        pos += 1; // skip '='
        let value = read_value(bytes, &mut pos)?;
        map.insert(key, coerce_value(value));
    }
    Ok(map)
}

fn read_value(bytes: &[u8], pos: &mut usize) -> Result<String> {
    if *pos >= bytes.len() {
        return value_err(*pos, "missing value after '='");
    }
    if bytes[*pos] == b'"' {
        *pos += 1;
        let start = *pos;
        let mut out = String::new();
        while *pos < bytes.len() {
            let b = bytes[*pos];
            if b == b'\\' {
                *pos += 1;
                if *pos >= bytes.len() {
                    return value_err(*pos, "unterminated escape in quoted value");
                }
                out.push(match bytes[*pos] {
                    b'"' => '"',
                    b'\\' => '\\',
                    b'n' => '\n',
                    b'r' => '\r',
                    b't' => '\t',
                    other => char::from(other),
                });
                *pos += 1;
            } else if b == b'"' {
                *pos += 1;
                while *pos < bytes.len() && bytes[*pos].is_ascii_whitespace() {
                    *pos += 1;
                }
                return Ok(out);
            } else {
                out.push(char::from(b));
                *pos += 1;
            }
        }
        let _ = start;
        return value_err(*pos, "unterminated quoted value");
    }
    let start = *pos;
    while *pos < bytes.len() && !bytes[*pos].is_ascii_whitespace() {
        *pos += 1;
    }
    std::str::from_utf8(&bytes[start..*pos])
        .map(|s| s.to_string())
        .map_err(|_| KvfmtError::LogfmtParse {
            pos: start,
            message: "invalid UTF-8 in value".into(),
        })
}

fn coerce_value(raw: String) -> Value {
    if raw == "true" {
        return Value::Bool(true);
    }
    if raw == "false" {
        return Value::Bool(false);
    }
    if raw == "null" {
        return Value::Null;
    }
    if let Ok(n) = raw.parse::<i64>() {
        return Value::Number(n.into());
    }
    if let Ok(n) = raw.parse::<f64>() {
        if let Some(num) = serde_json::Number::from_f64(n) {
            return Value::Number(num);
        }
    }
    Value::String(raw)
}

pub fn stringify(map: &Map<String, Value>) -> Result<String> {
    let mut keys: Vec<_> = map.keys().collect();
    keys.sort();
    let mut parts = Vec::with_capacity(keys.len());
    for key in keys {
        let value = map.get(key).expect("key exists");
        parts.push(encode_pair(key, value)?);
    }
    Ok(parts.join(" "))
}

fn encode_pair(key: &str, value: &Value) -> Result<String> {
    match value {
        Value::Bool(true) => Ok(key.to_string()),
        Value::Bool(false) => Ok(format!("{key}=false")),
        Value::Null => Ok(format!("{key}=null")),
        Value::Number(n) => Ok(format!("{key}={n}")),
        Value::String(s) => Ok(format!("{key}={}", encode_string(s))),
        Value::Object(_) | Value::Array(_) => Err(KvfmtError::UnsupportedValue {
            key: key.to_string(),
        }),
    }
}

fn encode_string(s: &str) -> String {
    let needs_quote = s.is_empty()
        || s.chars()
            .any(|c| c.is_ascii_whitespace() || c == '=' || c == '"');
    if !needs_quote {
        return s.to_string();
    }
    let mut out = String::from("\"");
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn parse_err(pos: usize, message: &str) -> Result<Map<String, Value>> {
    Err(KvfmtError::LogfmtParse {
        pos,
        message: message.to_string(),
    })
}

fn value_err(pos: usize, message: &str) -> Result<String> {
    Err(KvfmtError::LogfmtParse {
        pos,
        message: message.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_example_line() {
        let m = parse(r#"foo=bar a=14 baz="hello kitty" cool%story=bro f %^asdf"#).unwrap();
        assert_eq!(m["foo"], "bar");
        assert_eq!(m["a"], 14);
        assert_eq!(m["baz"], "hello kitty");
        assert_eq!(m["cool%story"], "bro");
        assert_eq!(m["f"], true);
        assert_eq!(m["%^asdf"], true);
    }

    #[test]
    fn stringify_example() {
        let mut m = Map::new();
        m.insert("foo".into(), Value::String("bar".into()));
        m.insert("a".into(), Value::Number(14.into()));
        m.insert("baz".into(), Value::String("hello kitty".into()));
        assert_eq!(stringify(&m).unwrap(), r#"a=14 baz="hello kitty" foo=bar"#);
    }
}
