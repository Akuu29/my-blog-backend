//! Application-level error type for the blog driver layer
//!
//! This module provides the top-level AppError type that aggregates all
//! errors from lower layers and implements HTTP response conversion.

use super::response::{ErrorCode, ErrorResponse};
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use blog_adapter::utils::repository_error::RepositoryError;
use blog_app::service::{
    articles::ArticleUsecaseError, categories::CategoryUsecaseError, comments::CommentUsecaseError,
    images::ImageUsecaseError, tags::TagUsecaseError, tokens::error::TokenServiceError,
    users::UserUsecaseError,
};
use blog_domain::error::{ErrorCategory, ErrorMetadata};
use thiserror::Error;

/// Top-level application error
///
/// Aggregates all errors from domain, application, and adapter layers.
/// Implements IntoResponse for automatic HTTP response conversion.
#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum AppError {
    #[error("ValidationFailed: [{0}]")]
    ValidationFailed(String),

    #[error("Unexpected: [{0}]")]
    Unexpected(String),

    #[error(transparent)]
    User(#[from] UserUsecaseError),

    #[error(transparent)]
    Token(#[from] TokenServiceError),

    #[error(transparent)]
    Image(#[from] ImageUsecaseError),

    #[error(transparent)]
    Comment(#[from] CommentUsecaseError),

    #[error(transparent)]
    Article(#[from] ArticleUsecaseError),

    #[error(transparent)]
    Category(#[from] CategoryUsecaseError),

    #[error(transparent)]
    Tag(#[from] TagUsecaseError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

/// Convert ErrorCategory to HTTP StatusCode
///
/// This is the ONLY place where domain concepts are mapped to HTTP.
/// The domain layer remains infrastructure-independent.
fn error_category_to_status(category: ErrorCategory) -> StatusCode {
    match category {
        ErrorCategory::Authentication => StatusCode::UNAUTHORIZED,
        ErrorCategory::Authorization => StatusCode::FORBIDDEN,
        ErrorCategory::Validation => StatusCode::BAD_REQUEST,
        ErrorCategory::NotFound => StatusCode::NOT_FOUND,
        ErrorCategory::Conflict => StatusCode::CONFLICT,
        ErrorCategory::Database => StatusCode::INTERNAL_SERVER_ERROR,
        ErrorCategory::ExternalService => StatusCode::BAD_GATEWAY,
        ErrorCategory::Internal => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// Convert ErrorCategory to client-facing ErrorCode
fn error_category_to_code(category: ErrorCategory) -> ErrorCode {
    match category {
        ErrorCategory::Authentication => ErrorCode::Unauthorized,
        ErrorCategory::Authorization => ErrorCode::Forbidden,
        ErrorCategory::Validation => ErrorCode::ValidationError,
        ErrorCategory::NotFound => ErrorCode::NotFound,
        ErrorCategory::Conflict => ErrorCode::Conflict,
        ErrorCategory::Database => ErrorCode::DatabaseError,
        ErrorCategory::ExternalService => ErrorCode::ExternalServiceError,
        ErrorCategory::Internal => ErrorCode::InternalError,
    }
}

/// Extract error metadata from any ErrorMetadata implementor
fn extract_error_info<E: ErrorMetadata>(error: &E) -> (StatusCode, ErrorCode, String) {
    let category = error.error_category();
    let status = error_category_to_status(category);
    let code = error_category_to_code(category);
    let message = error.user_message();

    (status, code, message)
}

/// Axum IntoResponse implementation for automatic HTTP response conversion
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
            // Handle non-ErrorMetadata errors
            Self::ValidationFailed(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorCode::ValidationError,
                msg.clone(),
            ),
            Self::Unexpected(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::InternalError,
                format!("An unexpected error occurred: {}", msg),
            ),

            // All ErrorMetadata implementations use unified extraction
            Self::User(e) => extract_error_info(e),
            Self::Token(e) => extract_error_info(e),
            Self::Image(e) => extract_error_info(e),
            Self::Comment(e) => extract_error_info(e),
            Self::Article(e) => extract_error_info(e),
            Self::Category(e) => extract_error_info(e),
            Self::Tag(e) => extract_error_info(e),
            Self::Repository(e) => extract_error_info(e),
        };

        // Log the error (will be enhanced by middleware in production)
        tracing::error!(
            error.message = %self,
            error.code = ?error_code,
            status = %status,
            "Request failed"
        );

        let error_response = ErrorResponse::new(error_code, message);
        let body = Json(error_response);

        (status, body).into_response()
    }
}

/// Convert anyhow::Error to AppError with simplified downcasting
impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        // Try to downcast to known error types
        if let Some(err) = e.downcast_ref::<UserUsecaseError>() {
            return Self::Unexpected(err.to_string());
        }
        if let Some(err) = e.downcast_ref::<TokenServiceError>() {
            return Self::Unexpected(err.to_string());
        }
        if let Some(err) = e.downcast_ref::<ImageUsecaseError>() {
            return Self::Unexpected(err.to_string());
        }
        if let Some(err) = e.downcast_ref::<CommentUsecaseError>() {
            return Self::Unexpected(err.to_string());
        }
        if let Some(err) = e.downcast_ref::<ArticleUsecaseError>() {
            return Self::Unexpected(err.to_string());
        }
        if let Some(err) = e.downcast_ref::<CategoryUsecaseError>() {
            return Self::Unexpected(err.to_string());
        }
        if let Some(err) = e.downcast_ref::<TagUsecaseError>() {
            return Self::Unexpected(err.to_string());
        }
        if let Some(err) = e.downcast_ref::<RepositoryError>() {
            return Self::Unexpected(err.to_string());
        }

        // Fallback for unknown errors
        Self::Unexpected(e.to_string())
    }
}
