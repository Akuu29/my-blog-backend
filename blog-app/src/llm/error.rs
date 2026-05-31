use blog_domain::error::ErrorCategory;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SummaryGeneratorError {
    #[error("Rate limit exceeded")]
    RateLimit { retry_after: Option<Duration> },
    #[error("Content policy violation")]
    ContentPolicyViolation,
    // In the future, this will handle errors related to the context length.
    #[error("Context length exceeded")]
    ContextLengthExceeded,
    #[error("Request timed out")]
    Timeout,
    #[error("API error: {0}")]
    ApiError(String),
}

impl SummaryGeneratorError {
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::RateLimit { .. } => ErrorCategory::RateLimit,
            Self::ContentPolicyViolation => ErrorCategory::ContentPolicy,
            Self::ContextLengthExceeded => ErrorCategory::ContentPolicy,
            Self::Timeout => ErrorCategory::ExternalService,
            Self::ApiError(_) => ErrorCategory::ExternalService,
        }
    }
}
