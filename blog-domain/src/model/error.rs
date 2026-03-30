use crate::error::ErrorCategory;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Resource not found")]
    NotFound,
    #[error("Resource already exists")]
    Conflict,
    #[error("Unexpected error: {0}")]
    Unknown(Box<dyn std::error::Error + Send + Sync>),
}

impl RepositoryError {
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::NotFound => ErrorCategory::NotFound,
            Self::Conflict => ErrorCategory::Conflict,
            Self::Unknown(_) => ErrorCategory::Internal,
        }
    }
}
