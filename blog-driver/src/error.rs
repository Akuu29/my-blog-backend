use crate::model::error_response::{ErrorCode, ErrorResponse};
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use blog_app::{
    query_service::error::QueryServiceError,
    service::{error::UsecaseError, tokens::error::TokenServiceError},
};
use blog_domain::error::ErrorCategory;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("ValidationFailed: [{0}]")]
    ValidationFailed(String),

    #[error("Unexpected error")]
    Unknown(#[from] anyhow::Error),

    #[error(transparent)]
    Usecase(#[from] UsecaseError),

    #[error(transparent)]
    Token(#[from] TokenServiceError),

    #[error(transparent)]
    QueryService(#[from] QueryServiceError),
}

impl AppError {
    fn category(&self) -> ErrorCategory {
        match self {
            Self::ValidationFailed(_) => ErrorCategory::Validation,
            Self::Unknown(_) => ErrorCategory::Internal,
            Self::Usecase(e) => e.category(),
            Self::Token(e) => e.category(),
            Self::QueryService(e) => e.category(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let category = self.category();

        match category {
            ErrorCategory::Internal => {
                tracing::error!(error = ?self, "Unexpected error occurred");
            }
            ErrorCategory::Authentication
            | ErrorCategory::Authorization
            | ErrorCategory::RateLimit
            | ErrorCategory::ExternalService => {
                tracing::warn!(error = %self, category = %category, "Request failed");
            }
            ErrorCategory::NotFound
            | ErrorCategory::Validation
            | ErrorCategory::Conflict
            | ErrorCategory::ContentPolicy => {
                tracing::info!(error = %self, category = %category, "Request failed");
            }
        }

        let (status, error_code, message) = match category {
            ErrorCategory::NotFound => (
                StatusCode::NOT_FOUND,
                ErrorCode::NotFound,
                "Resource not found",
            ),
            ErrorCategory::Authorization => {
                (StatusCode::FORBIDDEN, ErrorCode::Forbidden, "Forbidden")
            }
            ErrorCategory::Authentication => (
                StatusCode::UNAUTHORIZED,
                ErrorCode::Unauthorized,
                "Unauthorized",
            ),
            ErrorCategory::Validation => (
                StatusCode::BAD_REQUEST,
                ErrorCode::ValidationError,
                "Validation failed",
            ),
            ErrorCategory::Conflict => (StatusCode::CONFLICT, ErrorCode::Conflict, "Conflict"),
            ErrorCategory::RateLimit => (
                StatusCode::TOO_MANY_REQUESTS,
                ErrorCode::ServiceUnavailable,
                "Summary generation is temporarily unavailable. Please try again later.",
            ),
            ErrorCategory::ExternalService => (
                StatusCode::BAD_GATEWAY,
                ErrorCode::ExternalServiceError,
                "External service error. Please try again later.",
            ),
            ErrorCategory::ContentPolicy => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ErrorCode::InvalidInput,
                "The article content could not be processed.",
            ),
            ErrorCategory::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::InternalError,
                "Internal server error",
            ),
        };

        let error_response = ErrorResponse::new(error_code, message.to_string());
        (status, Json(error_response)).into_response()
    }
}
