use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("invalid input: {0}")]
    InvalidInput(&'static str),
    #[error("serialization error: {0}")]
    Serialization(String),
}
