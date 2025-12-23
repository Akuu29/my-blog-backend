use blog_domain::error::{ErrorCategory, ErrorMetadata, ErrorSeverity};
use blog_domain::service::users::UserServiceError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserUsecaseError {
    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error(transparent)]
    DomainError(#[from] UserServiceError),
}

impl ErrorMetadata for UserUsecaseError {
    fn error_category(&self) -> ErrorCategory {
        match self {
            Self::RepositoryError(_) => ErrorCategory::Database,
            Self::DomainError(e) => e.error_category(), // Delegate to domain error
        }
    }

    fn severity(&self) -> ErrorSeverity {
        match self {
            Self::RepositoryError(_) => ErrorSeverity::Error,
            Self::DomainError(e) => e.severity(), // Delegate to domain error
        }
    }

    fn user_message(&self) -> String {
        match self {
            Self::RepositoryError(_) => {
                "A database error occurred. Please try again later.".to_string()
            }
            Self::DomainError(e) => e.user_message(), // Delegate to domain error
        }
    }

    fn internal_context(&self) -> Option<String> {
        match self {
            Self::RepositoryError(msg) => Some(msg.clone()),
            Self::DomainError(e) => e.internal_context(), // Delegate to domain error
        }
    }
}
