use thiserror::Error;

#[derive(Debug, Error)]
pub enum KvfmtError {
    #[error("invalid logfmt at byte {pos}: {message}")]
    LogfmtParse { pos: usize, message: String },
    #[error("JSON must be a flat object with string keys")]
    NotFlatObject,
    #[error("unsupported JSON value at key `{key}`: nested objects and arrays are not supported")]
    UnsupportedValue { key: String },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, KvfmtError>;
