use blog_domain::error::{ErrorCategory, ErrorMetadata, ErrorSeverity};
use blog_domain::service::comments::CommentServiceError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommentUsecaseError {
    #[error("Repository error: {0}")]
    RepositoryError(String),
    #[error(transparent)]
    DomainError(#[from] CommentServiceError),
}

impl ErrorMetadata for CommentUsecaseError {
    fn error_category(&self) -> ErrorCategory {
        match self {
            Self::RepositoryError(_) => ErrorCategory::Database,
            Self::DomainError(e) => e.error_category(),
        }
    }

    fn severity(&self) -> ErrorSeverity {
        match self {
            Self::RepositoryError(_) => ErrorSeverity::Error,
            Self::DomainError(e) => e.severity(),
        }
    }

    fn user_message(&self) -> String {
        match self {
            Self::RepositoryError(_) => {
                "A database error occurred. Please try again later.".to_string()
            }
            Self::DomainError(e) => e.user_message(),
        }
    }

    fn internal_context(&self) -> Option<String> {
        match self {
            Self::RepositoryError(msg) => Some(msg.clone()),
            Self::DomainError(e) => e.internal_context(),
        }
    }
}
