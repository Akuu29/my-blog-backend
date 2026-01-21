use blog_domain::error::{ErrorCategory, ErrorMetadata, ErrorSeverity};
use blog_domain::service::images::ImageServiceError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImageUsecaseError {
    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("Image not found")]
    NotFound,

    #[error(transparent)]
    DomainError(#[from] ImageServiceError),
}

impl ErrorMetadata for ImageUsecaseError {
    fn error_category(&self) -> ErrorCategory {
        match self {
            Self::RepositoryError(_) => ErrorCategory::Database,
            Self::NotFound => ErrorCategory::NotFound,
            Self::DomainError(e) => e.error_category(),
        }
    }

    fn severity(&self) -> ErrorSeverity {
        match self {
            Self::RepositoryError(_) => ErrorSeverity::Error,
            Self::NotFound => ErrorSeverity::Info,
            Self::DomainError(e) => e.severity(),
        }
    }

    fn user_message(&self) -> String {
        match self {
            Self::RepositoryError(_) => {
                "A database error occurred. Please try again later.".to_string()
            }
            Self::NotFound => "The requested image was not found.".to_string(),
            Self::DomainError(e) => e.user_message(),
        }
    }

    fn internal_context(&self) -> Option<String> {
        match self {
            Self::RepositoryError(msg) => Some(msg.clone()),
            Self::NotFound => None,
            Self::DomainError(e) => e.internal_context(),
        }
    }
}
