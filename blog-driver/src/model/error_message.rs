use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorMessage {
    kind: ErrorMessageKind,
    message: String,
}

impl ErrorMessage {
    pub fn new(kind: ErrorMessageKind, message: String) -> Self {
        Self { kind, message }
    }
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub enum ErrorMessageKind {
    // Failed to authenticate
    Unauthorized,
    // Failed to authorize
    Forbidden,
    // Validation failed
    Validation,
    // Resource not found
    NotFound,
    // Resource already exists
    Conflict,
    // Internal server error
    Internal,
    // Bad request
    BadRequest,
    // Service timeout
    Timeout,
    // Service unavailable
    ServiceUnavailable,
}
