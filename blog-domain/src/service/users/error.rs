use crate::error::{ErrorCategory, ErrorMetadata, ErrorSeverity};

#[derive(Debug)]
pub enum UserServiceError {
    Unauthorized,
    InternalError(String),
}

impl std::fmt::Display for UserServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserServiceError::Unauthorized => {
                write!(f, "User is not authorized to perform this action")
            }
            UserServiceError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for UserServiceError {}

impl ErrorMetadata for UserServiceError {
    fn error_category(&self) -> ErrorCategory {
        match self {
            Self::Unauthorized => ErrorCategory::Authorization,
            Self::InternalError(_) => ErrorCategory::Internal,
        }
    }

    fn severity(&self) -> ErrorSeverity {
        match self {
            Self::Unauthorized => ErrorSeverity::Info,
            Self::InternalError(_) => ErrorSeverity::Error,
        }
    }

    fn user_message(&self) -> String {
        match self {
            Self::Unauthorized => "You can only access your own account".to_string(),
            Self::InternalError(_) => {
                "An internal error occurred. Please try again later.".to_string()
            }
        }
    }

    fn internal_context(&self) -> Option<String> {
        match self {
            Self::Unauthorized => None,
            Self::InternalError(msg) => Some(msg.clone()),
        }
    }
}
