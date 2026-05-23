mod error;
mod json_fmt;
mod logfmt;

pub use error::{KvfmtError, Result};
pub use json_fmt::parse_flat_object;
pub use logfmt::{parse as parse_logfmt, stringify as stringify_logfmt};

pub fn logfmt_to_json(input: &str, pretty: bool) -> Result<String> {
    let map = parse_logfmt(input)?;
    json_fmt::to_json_string(&map, pretty)
}

pub fn json_to_logfmt(input: &str) -> Result<String> {
    let map = parse_flat_object(input)?;
    stringify_logfmt(&map)
}
