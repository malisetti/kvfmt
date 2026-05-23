#![forbid(unsafe_code)]

pub mod parser {
    use serde_json::{Map, Value};

    pub fn parse_json(input: &str) -> Result<Vec<(String, String)>, String> {
        let v: Value = serde_json::from_str(input.trim()).map_err(|e| e.to_string())?;
        let obj = v.as_object().ok_or("expected JSON object")?;
        Ok(obj.iter().map(|(k, v)| (k.clone(), value_to_str(v))).collect())
    }

    fn value_to_str(v: &Value) -> String {
        match v {
            Value::String(s) => s.clone(),
            Value::Null => "null".into(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            _ => serde_json::to_string(v).unwrap_or_default(),
        }
    }

    pub fn parse_logfmt(input: &str) -> Result<Vec<(String, String)>, String> {
        let mut pairs = Vec::new();
        let b = input.trim().as_bytes();
        let mut i = 0usize;
        while i < b.len() {
            while i < b.len() && b[i].is_ascii_whitespace() {
                i += 1;
            }
            if i >= b.len() {
                break;
            }
            let key_start = i;
            while i < b.len() && b[i] != b'=' && !b[i].is_ascii_whitespace() {
                i += 1;
            }
            if i >= b.len() || b[i] != b'=' {
                return Err("expected key=value".into());
            }
            let key = std::str::from_utf8(&b[key_start..i])
                .map_err(|e| e.to_string())?
                .to_string();
            i += 1;
            let val = if i < b.len() && b[i] == b'"' {
                i += 1;
                let mut out = String::new();
                while i < b.len() {
                    if b[i] == b'\\' && i + 1 < b.len() {
                        i += 1;
                        out.push(match b[i] {
                            b'n' => '\n',
                            b'r' => '\r',
                            b't' => '\t',
                            b'"' => '"',
                            b'\\' => '\\',
                            c => c as char,
                        });
                        i += 1;
                    } else if b[i] == b'"' {
                        break;
                    } else {
                        out.push(b[i] as char);
                        i += 1;
                    }
                }
                if i >= b.len() || b[i] != b'"' {
                    return Err("unterminated quoted value".into());
                }
                i += 1;
                out
            } else {
                let start = i;
                while i < b.len() && !b[i].is_ascii_whitespace() {
                    i += 1;
                }
                std::str::from_utf8(&b[start..i])
                    .map_err(|e| e.to_string())?
                    .to_string()
            };
            pairs.push((key, val));
        }
        Ok(pairs)
    }

    pub fn emit_logfmt(pairs: &[(String, String)]) -> String {
        let mut out = String::new();
        for (i, (k, v)) in pairs.iter().enumerate() {
            if i > 0 {
                out.push(' ');
            }
            out.push_str(k);
            out.push('=');
            if v.is_empty()
                || v.contains(' ')
                || v.contains('"')
                || v.contains('\\')
                || v.contains('\n')
                || v.contains('\r')
                || v.contains('\t')
            {
                out.push('"');
                for c in v.chars() {
                    match c {
                        '\n' => out.push_str("\\n"),
                        '\r' => out.push_str("\\r"),
                        '\t' => out.push_str("\\t"),
                        '"' => out.push_str("\\\""),
                        '\\' => out.push_str("\\\\"),
                        c => out.push(c),
                    }
                }
                out.push('"');
            } else {
                out.push_str(v);
            }
        }
        out.push('\n');
        out
    }

    pub fn emit_json(pairs: &[(String, String)]) -> Result<String, String> {
        let mut map = Map::new();
        for (k, v) in pairs {
            map.insert(k.clone(), Value::String(v.clone()));
        }
        serde_json::to_string(&Value::Object(map)).map_err(|e| e.to_string())
    }
}
