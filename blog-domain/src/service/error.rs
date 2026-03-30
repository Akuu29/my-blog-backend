use crate::{error::ErrorCategory, model::error::RepositoryError};

#[derive(Debug, thiserror::Error)]
pub enum DomainServiceError {
    #[error("Resource not found")]
    NotFound,
    #[error("User is not authorized")]
    Unauthorized,
    #[error("Resource already exists")]
    Conflict,
    #[error("Unexpected error: {0}")]
    Unknown(Box<dyn std::error::Error + Send + Sync>),
}

impl DomainServiceError {
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::NotFound => ErrorCategory::NotFound,
            Self::Unauthorized => ErrorCategory::Authorization,
            Self::Conflict => ErrorCategory::Conflict,
            Self::Unknown(_) => ErrorCategory::Internal,
        }
    }
}

impl From<RepositoryError> for DomainServiceError {
    fn from(e: RepositoryError) -> Self {
        match e {
            RepositoryError::NotFound => Self::NotFound,
            RepositoryError::Conflict => Self::Conflict,
            RepositoryError::Unknown(e) => Self::Unknown(e),
        }
    }
}
