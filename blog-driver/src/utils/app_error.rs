use crate::{model::error_message::ErrorMessageKind, utils::error_log_kind::ErrorLogKind};
use axum::http::StatusCode;
use blog_adapter::utils::repository_error::RepositoryError;
use blog_app::utils::usecase_error::UsecaseError;
use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum AppError {
    #[error("ValidationFailed: [{0}]")]
    ValidationFailed(String),
    #[error("Unexpected: [{0}]")]
    Unexpected(String),
    #[error(transparent)]
    UsecaseError(#[from] UsecaseError),
    #[error(transparent)]
    RepositoryError(#[from] RepositoryError),
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            AppError::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::UsecaseError(UsecaseError::AuthenticationFailed(_)) => {
                StatusCode::UNAUTHORIZED
            }
            AppError::UsecaseError(UsecaseError::ValidationFailed(_)) => StatusCode::BAD_REQUEST,
            AppError::RepositoryError(RepositoryError::NotFound) => StatusCode::NOT_FOUND,
            AppError::RepositoryError(RepositoryError::Unexpected(_)) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    pub fn error_message_kind(&self) -> ErrorMessageKind {
        match self {
            AppError::ValidationFailed(_) => ErrorMessageKind::Validation,
            AppError::Unexpected(_) => ErrorMessageKind::Internal,
            AppError::UsecaseError(UsecaseError::AuthenticationFailed(_)) => {
                ErrorMessageKind::Unauthorized
            }
            AppError::UsecaseError(UsecaseError::ValidationFailed(_)) => {
                ErrorMessageKind::Validation
            }
            AppError::RepositoryError(RepositoryError::NotFound) => ErrorMessageKind::NotFound,
            AppError::RepositoryError(RepositoryError::Unexpected(_)) => ErrorMessageKind::Internal,
        }
    }

    pub fn error_log_kind(&self) -> ErrorLogKind {
        match self {
            AppError::ValidationFailed(_) => ErrorLogKind::Validation,
            AppError::Unexpected(_) => ErrorLogKind::Unexpected,
            AppError::UsecaseError(UsecaseError::AuthenticationFailed(_)) => {
                ErrorLogKind::Authentication
            }
            AppError::UsecaseError(UsecaseError::ValidationFailed(_)) => ErrorLogKind::Validation,
            AppError::RepositoryError(_) => ErrorLogKind::Database,
        }
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        match e.downcast::<AppError>() {
            Ok(app_error) => app_error,
            Err(e) => AppError::Unexpected(e.to_string()),
        }
    }
}
