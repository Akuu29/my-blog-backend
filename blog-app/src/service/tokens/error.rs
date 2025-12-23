use blog_domain::error::{ErrorCategory, ErrorMetadata, ErrorSeverity};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TokenServiceError {
    /// Token has expired
    #[error("Token has expired")]
    TokenExpired,
    #[error("Invalid token signature")]
    InvalidSignature,
    #[error("Invalid token format")]
    InvalidToken,
    #[error("Token validation failed: {0}")]
    ValidationFailed(String),
    #[error("Repository error: {0}")]
    RepositoryError(String),
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<jsonwebtoken::errors::Error> for TokenServiceError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        use jsonwebtoken::errors::ErrorKind;

        match err.kind() {
            // Authentication errors (401)
            ErrorKind::ExpiredSignature => Self::TokenExpired,
            ErrorKind::InvalidToken => Self::InvalidToken,
            ErrorKind::InvalidSignature => Self::InvalidSignature,

            // Validation errors (401)
            ErrorKind::InvalidIssuer => Self::ValidationFailed("Invalid issuer".to_string()),
            ErrorKind::InvalidAudience => Self::ValidationFailed("Invalid audience".to_string()),
            ErrorKind::InvalidSubject => Self::ValidationFailed("Invalid subject".to_string()),
            ErrorKind::ImmatureSignature => {
                Self::ValidationFailed("Token not yet valid".to_string())
            }

            // Internal errors (500)
            ErrorKind::InvalidEcdsaKey
            | ErrorKind::InvalidRsaKey(_)
            | ErrorKind::RsaFailedSigning
            | ErrorKind::InvalidKeyFormat => Self::InternalError(format!("Key error: {}", err)),

            // Catch-all for other errors
            _ => Self::InvalidToken,
        }
    }
}

impl ErrorMetadata for TokenServiceError {
    fn error_category(&self) -> ErrorCategory {
        match self {
            // Authentication errors → 401
            Self::TokenExpired
            | Self::InvalidSignature
            | Self::InvalidToken
            | Self::ValidationFailed(_) => ErrorCategory::Authentication,

            // Database errors → 500
            Self::RepositoryError(_) => ErrorCategory::Database,

            // Internal errors → 500
            Self::InternalError(_) => ErrorCategory::Internal,
        }
    }

    fn severity(&self) -> ErrorSeverity {
        match self {
            // User errors (expected)
            Self::TokenExpired
            | Self::InvalidSignature
            | Self::InvalidToken
            | Self::ValidationFailed(_) => ErrorSeverity::Info,

            // Server errors (unexpected)
            Self::RepositoryError(_) | Self::InternalError(_) => ErrorSeverity::Error,
        }
    }

    fn user_message(&self) -> String {
        match self {
            Self::TokenExpired => "Your session has expired. Please log in again.".to_string(),
            Self::InvalidSignature => {
                "Invalid authentication token. Please log in again.".to_string()
            }
            Self::InvalidToken => "Invalid authentication token. Please log in again.".to_string(),
            Self::ValidationFailed(_) => "Authentication failed. Please log in again.".to_string(),
            Self::RepositoryError(_) => {
                "A database error occurred. Please try again later.".to_string()
            }
            Self::InternalError(_) => {
                "An internal error occurred. Please try again later.".to_string()
            }
        }
    }

    fn internal_context(&self) -> Option<String> {
        match self {
            Self::TokenExpired | Self::InvalidSignature | Self::InvalidToken => None,

            Self::ValidationFailed(msg) | Self::RepositoryError(msg) | Self::InternalError(msg) => {
                Some(msg.clone())
            }
        }
    }
}
