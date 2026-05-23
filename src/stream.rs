//! Line-at-a-time stdin/stdout processing with flush after each emit.

use crate::logfmt;
use serde_json::{Map, Value};
use std::io::{self, BufRead, BufReader, Write};

pub fn json_lines_to_logfmt<R: BufRead, W: Write>(mut reader: R, mut writer: W) -> Result<u64, String> {
    let mut count = 0u64;
    let mut line_buf = String::new();
    loop {
        line_buf.clear();
        let n = reader.read_line(&mut line_buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        let trimmed = line_buf.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(trimmed).map_err(|e| format!("invalid JSON: {e}"))?;
        let map = value
            .as_object()
            .ok_or_else(|| "JSON value must be an object".to_string())?;
        let out = logfmt::encode_object(map)?;
        writeln!(writer, "{out}").map_err(|e| e.to_string())?;
        writer.flush().map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok(count)
}

pub fn logfmt_lines_to_json<R: BufRead, W: Write>(mut reader: R, mut writer: W) -> Result<u64, String> {
    let mut count = 0u64;
    let mut line_buf = String::new();
    loop {
        line_buf.clear();
        let n = reader.read_line(&mut line_buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        let trimmed = line_buf.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            continue;
        }
        let map: Map<String, Value> = logfmt::parse_line(trimmed)?;
        let out = serde_json::to_string(&Value::Object(map)).map_err(|e| e.to_string())?;
        writeln!(writer, "{out}").map_err(|e| e.to_string())?;
        writer.flush().map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok(count)
}

pub fn process_reader(mode: Mode, reader: impl BufRead, writer: impl Write) -> Result<u64, String> {
    match mode {
        Mode::JsonToLogfmt => json_lines_to_logfmt(reader, writer),
        Mode::LogfmtToJson => logfmt_lines_to_json(reader, writer),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    JsonToLogfmt,
    LogfmtToJson,
}

pub fn open_input(path: Option<&str>) -> Result<Box<dyn BufRead>, String> {
    match path {
        None | Some("-") => Ok(Box::new(BufReader::new(io::stdin().lock()))),
        Some(p) => {
            let f = std::fs::File::open(p).map_err(|e| format!("open {p}: {e}"))?;
            Ok(Box::new(BufReader::new(f)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn streams_multiple_json_lines() {
        let input = b"{\"a\":1}\n{\"b\":2}\n";
        let mut out = Vec::new();
        let n = json_lines_to_logfmt(Cursor::new(input), &mut out).unwrap();
        assert_eq!(n, 2);
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains("a=1"));
        assert!(s.contains("b=2"));
    }

    #[test]
    fn streams_logfmt_to_json() {
        let input = b"foo=bar\nx=1\n";
        let mut out = Vec::new();
        let n = logfmt_lines_to_json(Cursor::new(input), &mut out).unwrap();
        assert_eq!(n, 2);
        let text = String::from_utf8(out).unwrap();
        let lines: Vec<_> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("\"foo\""));
    }
}
