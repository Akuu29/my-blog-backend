use blog_domain::error::ErrorCategory;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TokenServiceError {
    #[error("Token has expired")]
    TokenExpired,
    #[error("Invalid token signature")]
    InvalidSignature,
    #[error("Invalid token format")]
    InvalidToken,
    #[error("Token validation failed: {0}")]
    ValidationFailed(String),
    #[error("Unexpected error")]
    Unknown(#[from] anyhow::Error),
}

impl From<jsonwebtoken::errors::Error> for TokenServiceError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        use jsonwebtoken::errors::ErrorKind;

        match err.kind() {
            ErrorKind::ExpiredSignature => Self::TokenExpired,
            ErrorKind::InvalidToken => Self::InvalidToken,
            ErrorKind::InvalidSignature => Self::InvalidSignature,
            ErrorKind::InvalidIssuer => Self::ValidationFailed("Invalid issuer".to_string()),
            ErrorKind::InvalidAudience => Self::ValidationFailed("Invalid audience".to_string()),
            ErrorKind::InvalidSubject => Self::ValidationFailed("Invalid subject".to_string()),
            ErrorKind::ImmatureSignature => {
                Self::ValidationFailed("Token not yet valid".to_string())
            }
            ErrorKind::InvalidEcdsaKey
            | ErrorKind::InvalidRsaKey(_)
            | ErrorKind::RsaFailedSigning
            | ErrorKind::InvalidKeyFormat => Self::Unknown(anyhow::anyhow!("Key error: {}", err)),
            _ => Self::InvalidToken,
        }
    }
}

impl TokenServiceError {
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::TokenExpired
            | Self::InvalidSignature
            | Self::InvalidToken
            | Self::ValidationFailed(_) => ErrorCategory::Authentication,
            Self::Unknown(_) => ErrorCategory::Internal,
        }
    }
}
