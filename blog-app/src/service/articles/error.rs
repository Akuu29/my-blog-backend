use blog_domain::{
    error::{ErrorCategory, ErrorMetadata, ErrorSeverity},
    service::articles::ArticleServiceError,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ArticleUsecaseError {
    #[error("Repository error: {0}")]
    RepositoryError(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    #[error(transparent)]
    DomainError(#[from] ArticleServiceError),
}

impl ErrorMetadata for ArticleUsecaseError {
    fn error_category(&self) -> ErrorCategory {
        match self {
            Self::RepositoryError(_) => ErrorCategory::Database,
            Self::ValidationFailed(_) => ErrorCategory::Validation,
            Self::DomainError(e) => e.error_category(),
        }
    }

    fn severity(&self) -> ErrorSeverity {
        match self {
            Self::RepositoryError(_) => ErrorSeverity::Error,
            Self::ValidationFailed(_) => ErrorSeverity::Info,
            Self::DomainError(e) => e.severity(),
        }
    }

    fn user_message(&self) -> String {
        match self {
            Self::RepositoryError(_) => {
                "A database error occurred. Please try again later.".to_string()
            }
            Self::ValidationFailed(msg) => msg.clone(),
            Self::DomainError(e) => e.user_message(),
        }
    }

    fn internal_context(&self) -> Option<String> {
        match self {
            Self::RepositoryError(msg) => Some(msg.clone()),
            Self::ValidationFailed(_) => None,
            Self::DomainError(e) => e.internal_context(),
        }
    }
}
