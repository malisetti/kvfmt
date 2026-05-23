//! logfmt parse and encode (kr/logfmt-style semantics).

use serde_json::{Map, Number, Value};
use std::collections::BTreeMap;

/// Parse one logfmt line into a JSON object (string values; bare keys → true).
pub fn parse_line(line: &str) -> Result<Map<String, Value>, String> {
    let mut out = Map::new();
    let bytes = line.as_bytes();
    let mut i = 0usize;
    let len = bytes.len();

    while i < len {
        // skip leading garbage (non-ident bytes)
        while i < len && !is_ident_byte(bytes[i]) {
            i += 1;
        }
        if i >= len {
            break;
        }

        let key_start = i;
        while i < len && is_ident_byte(bytes[i]) {
            i += 1;
        }
        let key = std::str::from_utf8(&bytes[key_start..i])
            .map_err(|e| e.to_string())?
            .to_string();

        if i < len && bytes[i] == b'=' {
            i += 1;
            if i >= len {
                out.insert(key, Value::Bool(true));
                break;
            }
            if bytes[i] == b'"' {
                i += 1;
                let val_start = i;
                let mut val = String::new();
                while i < len {
                    if bytes[i] == b'\\' && i + 1 < len {
                        i += 1;
                        val.push(bytes[i] as char);
                        i += 1;
                    } else if bytes[i] == b'"' {
                        i += 1;
                        break;
                    } else {
                        val.push(bytes[i] as char);
                        i += 1;
                    }
                }
                let _ = val_start; // consumed in loop
                out.insert(key, Value::String(val));
            } else {
                let val_start = i;
                while i < len && is_ident_byte(bytes[i]) {
                    i += 1;
                }
                let raw = std::str::from_utf8(&bytes[val_start..i]).map_err(|e| e.to_string())?;
                out.insert(key, parse_scalar(raw));
            }
        } else {
            out.insert(key, Value::Bool(true));
        }
    }

    Ok(out)
}

fn is_ident_byte(b: u8) -> bool {
    b > b' ' && b != b'=' && b != b'"'
}

fn parse_scalar(s: &str) -> Value {
    if s == "true" {
        Value::Bool(true)
    } else if s == "false" {
        Value::Bool(false)
    } else if let Ok(n) = s.parse::<i64>() {
        Value::Number(n.into())
    } else if let Ok(n) = s.parse::<f64>() {
        Number::from_f64(n).map(Value::Number).unwrap_or(Value::String(s.to_string()))
    } else {
        Value::String(s.to_string())
    }
}

/// Encode a JSON object as one logfmt line (sorted keys for stable output).
pub fn encode_object(map: &Map<String, Value>) -> Result<String, String> {
    let mut pairs: BTreeMap<&str, &Value> = BTreeMap::new();
    for (k, v) in map {
        if v.is_null() {
            continue;
        }
        pairs.insert(k.as_str(), v);
    }
    let mut parts = Vec::new();
    for (key, value) in pairs {
        parts.push(encode_pair(key, value)?);
    }
    Ok(parts.join(" "))
}

fn encode_pair(key: &str, value: &Value) -> Result<String, String> {
    if !key.bytes().all(is_ident_byte) {
        return Err(format!("invalid logfmt key: {key}"));
    }
    match value {
        Value::Bool(b) => Ok(format!("{key}={b}")),
        Value::Number(n) => Ok(format!("{key}={n}")),
        Value::String(s) => {
            if needs_quoting(s) {
                Ok(format!("{key}=\"{}\"", escape_string(s)))
            } else {
                Ok(format!("{key}={s}"))
            }
        }
        Value::Null => Ok(key.to_string()),
        other => {
            let s = serde_json::to_string(other).map_err(|e| e.to_string())?;
            Ok(format!("{key}=\"{}\"", escape_string(&s)))
        }
    }
}

fn needs_quoting(s: &str) -> bool {
    s.is_empty()
        || s.bytes().any(|b| !is_ident_byte(b))
        || s == "true"
        || s == "false"
}

fn escape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        if ch == '"' || ch == '\\' {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_roundtrip_example() {
        let line = r#"foo=bar a=14 baz="hello kitty" cool%story=bro f %^asdf"#;
        let map = parse_line(line).unwrap();
        assert_eq!(map.get("foo").unwrap(), &json!("bar"));
        assert_eq!(map.get("a").unwrap(), &json!(14));
        assert_eq!(map.get("baz").unwrap(), &json!("hello kitty"));
        assert_eq!(map.get("f").unwrap(), &json!(true));
    }

    #[test]
    fn encode_decode_symmetry() {
        let original = json!({"at": "info", "method": "GET", "status": 200});
        let map = original.as_object().unwrap();
        let line = encode_object(map).unwrap();
        let back = parse_line(&line).unwrap();
        assert_eq!(back.get("at").unwrap(), &json!("info"));
        assert_eq!(back.get("status").unwrap(), &json!(200));
    }
}
