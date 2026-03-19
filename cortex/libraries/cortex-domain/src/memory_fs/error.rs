use crate::DomainError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MemoryFsError {
    #[error("Object not found: {0}")]
    ObjectNotFound(String),
    #[error("Invalid object type: expected {expected}, found {found}")]
    InvalidObjectType { expected: String, found: String },
    #[error("Path not found: {0}")]
    PathNotFound(String),
    #[error("Reference not found: {0}")]
    RefNotFound(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Corrupt object: {0}")]
    CorruptObject(String),
}

impl From<MemoryFsError> for DomainError {
    fn from(err: MemoryFsError) -> Self {
        DomainError::InvariantViolation(err.to_string())
    }
}
