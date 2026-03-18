use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("domain error: {0}")]
    Domain(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("invalid timestamp: {0}")]
    InvalidTimestamp(u64),
}
