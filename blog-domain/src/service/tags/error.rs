use crate::error::{ErrorCategory, ErrorMetadata, ErrorSeverity};

#[derive(Debug)]
pub enum TagServiceError {
    NotFound,
    Unauthorized,
    InternalError(String),
}

impl std::fmt::Display for TagServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TagServiceError::NotFound => write!(f, "Tag not found"),
            TagServiceError::Unauthorized => {
                write!(f, "User is not authorized to access this tag")
            }
            TagServiceError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for TagServiceError {}

impl ErrorMetadata for TagServiceError {
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
            Self::NotFound => "Tag not found".to_string(),
            Self::Unauthorized => "Tag is not owned by user".to_string(),
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
