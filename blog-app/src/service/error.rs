use blog_domain::{
    error::ErrorCategory, model::error::RepositoryError, service::error::DomainServiceError,
};

#[derive(Debug, thiserror::Error)]
pub enum UsecaseError {
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    #[error(transparent)]
    DomainError(#[from] DomainServiceError),
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

impl UsecaseError {
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::ValidationFailed(_) => ErrorCategory::Validation,
            Self::DomainError(e) => e.category(),
            Self::Repository(e) => e.category(),
        }
    }
}
