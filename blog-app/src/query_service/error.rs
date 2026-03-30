use blog_domain::error::ErrorCategory;

#[derive(Debug, thiserror::Error)]
pub enum QueryServiceError {
    #[error("Invalid cursor: cursor ID does not exist")]
    InvalidCursor,
    #[error("Unexpected error: {0}")]
    Unknown(Box<dyn std::error::Error + Send + Sync>),
}

impl QueryServiceError {
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::InvalidCursor => ErrorCategory::Validation,
            Self::Unknown(_) => ErrorCategory::Internal,
        }
    }
}
