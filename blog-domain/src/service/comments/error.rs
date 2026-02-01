use crate::error::{ErrorCategory, ErrorMetadata, ErrorSeverity};

#[derive(Debug)]
pub enum CommentServiceError {
    NotFound,
    Unauthorized,
    InternalError(String),
}

impl std::fmt::Display for CommentServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommentServiceError::NotFound => write!(f, "Comment not found"),
            CommentServiceError::Unauthorized => {
                write!(f, "User is not authorized to access this comment")
            }
            CommentServiceError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for CommentServiceError {}

impl ErrorMetadata for CommentServiceError {
    fn error_category(&self) -> ErrorCategory {
        match self {
            Self::NotFound => ErrorCategory::NotFound,
            Self::Unauthorized => ErrorCategory::Authorization,
            Self::InternalError(_) => ErrorCategory::Internal,
        }
    }

    fn severity(&self) -> ErrorSeverity {
        match self {
            Self::NotFound => ErrorSeverity::Info,
            Self::Unauthorized => ErrorSeverity::Info,
            Self::InternalError(_) => ErrorSeverity::Error,
        }
    }

    fn user_message(&self) -> String {
        match self {
            Self::NotFound => "Comment not found".to_string(),
            Self::Unauthorized => "Comment is not owned by user".to_string(),
            Self::InternalError(_) => {
                "An internal error occurred. Please try again later.".to_string()
            }
        }
    }

    fn internal_context(&self) -> Option<String> {
        match self {
            Self::NotFound | Self::Unauthorized => None,
            Self::InternalError(msg) => Some(msg.clone()),
        }
    }
}
