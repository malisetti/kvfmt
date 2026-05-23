use crate::error::{KvfmtError, Result};
use serde_json::{Map, Value};

pub fn parse_flat_object(input: &str) -> Result<Map<String, Value>> {
    let value: Value = serde_json::from_str(input)?;
    match value {
        Value::Object(map) => {
            for (k, v) in &map {
                ensure_flat(k, v)?;
            }
            Ok(map)
        }
        _ => Err(KvfmtError::NotFlatObject),
    }
}

fn ensure_flat(key: &str, value: &Value) -> Result<()> {
    match value {
        Value::Object(_) | Value::Array(_) => Err(KvfmtError::UnsupportedValue {
            key: key.to_string(),
        }),
        _ => Ok(()),
    }
}

pub fn to_json_string(map: &Map<String, Value>, pretty: bool) -> Result<String> {
    if pretty {
        Ok(serde_json::to_string_pretty(map)?)
    } else {
        Ok(serde_json::to_string(map)?)
    }
}
