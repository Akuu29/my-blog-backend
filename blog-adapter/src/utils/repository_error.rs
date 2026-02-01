use blog_domain::error::{ErrorCategory, ErrorMetadata, ErrorSeverity};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Unexpected Error: [{0}]")]
    Unexpected(String),
    #[error("NotFound")]
    NotFound,
}

impl ErrorMetadata for RepositoryError {
    fn error_category(&self) -> ErrorCategory {
        match self {
            Self::NotFound => ErrorCategory::NotFound,
            Self::Unexpected(_) => ErrorCategory::Database,
        }
    }

    fn severity(&self) -> ErrorSeverity {
        match self {
            Self::NotFound => ErrorSeverity::Info,
            Self::Unexpected(_) => ErrorSeverity::Error,
        }
    }

    fn user_message(&self) -> String {
        match self {
            Self::NotFound => "The requested resource was not found".to_string(),
            Self::Unexpected(_) => "A database error occurred. Please try again later.".to_string(),
        }
    }

    fn internal_context(&self) -> Option<String> {
        match self {
            Self::NotFound => None,
            Self::Unexpected(msg) => Some(msg.clone()),
        }
    }
}
