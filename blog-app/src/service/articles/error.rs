use blog_domain::error::{ErrorCategory, ErrorMetadata, ErrorSeverity};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ArticleUsecaseError {
    #[error("Repository error: {0}")]
    RepositoryError(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

impl ErrorMetadata for ArticleUsecaseError {
    fn error_category(&self) -> ErrorCategory {
        match self {
            Self::RepositoryError(_) => ErrorCategory::Database,
            Self::PermissionDenied(_) => ErrorCategory::Authorization,
            Self::ValidationFailed(_) => ErrorCategory::Validation,
        }
    }

    fn severity(&self) -> ErrorSeverity {
        match self {
            Self::RepositoryError(_) => ErrorSeverity::Error,
            Self::PermissionDenied(_) => ErrorSeverity::Info,
            Self::ValidationFailed(_) => ErrorSeverity::Info,
        }
    }

    fn user_message(&self) -> String {
        match self {
            Self::RepositoryError(_) => {
                "A database error occurred. Please try again later.".to_string()
            }
            Self::PermissionDenied(msg) => msg.clone(),
            Self::ValidationFailed(msg) => msg.clone(),
        }
    }

    fn internal_context(&self) -> Option<String> {
        match self {
            Self::RepositoryError(msg) => Some(msg.clone()),
            Self::PermissionDenied(_) | Self::ValidationFailed(_) => None,
        }
    }
}
