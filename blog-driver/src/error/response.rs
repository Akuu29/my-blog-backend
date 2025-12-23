use serde::Serialize;

/// Error response structure sent to clients
///
/// This structure provides a consistent error format across all API endpoints.
/// It separates internal error details from client-safe messages.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl ErrorResponse {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            error: ErrorDetail {
                code,
                message: message.into(),
                field: None,
            },
            request_id: None,
        }
    }

    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.error.field = Some(field.into());
        self
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }
}

/// Detailed error information
#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    /// Error code for unique identification
    pub code: ErrorCode,
    /// User-friendly error message (safe for clients)
    pub message: String,
    /// Field name (for validation errors)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

/// Error codes for client-facing API responses
///
/// These codes provide a stable API contract for error handling.
/// Messages are intentionally generic to avoid exposing internal implementation details.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[allow(dead_code)]
pub enum ErrorCode {
    // Authentication & Authorization errors
    /// Authentication required or credentials invalid
    Unauthorized,
    /// Insufficient permissions
    Forbidden,
    /// Invalid or expired token
    InvalidToken,

    // Validation errors
    /// General validation error
    ValidationError,
    /// Invalid email format
    InvalidEmail,
    /// Invalid password format
    InvalidPassword,
    /// Invalid input data
    InvalidInput,

    // Resource errors
    /// Resource not found
    NotFound,
    /// Resource already exists (conflict)
    AlreadyExists,
    /// Conflict with current state
    Conflict,

    // Server errors
    /// Internal server error
    InternalError,
    /// Database operation failed
    DatabaseError,
    /// Service temporarily unavailable
    ServiceUnavailable,
    /// External service error
    ExternalServiceError,
}

impl ErrorCode {
    /// Returns a safe, user-friendly default message for this error code
    ///
    /// These messages do not expose internal implementation details or
    /// sensitive information. They provide actionable guidance to users.
    pub fn default_message(&self) -> &'static str {
        match self {
            // Authentication & Authorization
            Self::Unauthorized => "Authentication required. Please log in with valid credentials.",
            Self::Forbidden => "You don't have permission to access this resource.",
            Self::InvalidToken => "Your session has expired. Please log in again.",

            // Validation
            Self::ValidationError => "The provided data is invalid. Please check your input.",
            Self::InvalidEmail => "The email address format is invalid.",
            Self::InvalidPassword => "The password does not meet the required criteria.",
            Self::InvalidInput => "Invalid input data. Please check your request.",

            // Resources
            Self::NotFound => "The requested resource was not found.",
            Self::AlreadyExists => "A resource with these details already exists.",
            Self::Conflict => "The request conflicts with the current state of the resource.",

            // Server
            Self::InternalError => "An unexpected error occurred. Please try again later.",
            Self::DatabaseError => "A temporary error occurred. Please try again in a few moments.",
            Self::ServiceUnavailable => {
                "The service is temporarily unavailable. Please try again later."
            }
            Self::ExternalServiceError => {
                "An external service is unavailable. Please try again later."
            }
        }
    }
}
