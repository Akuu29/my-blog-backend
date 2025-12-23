//! Error handling foundation for the domain layer
//!
//! This module provides common error handling traits and types that are
//! independent of infrastructure concerns (HTTP, databases, etc.).
//!
//! # Architecture Principle
//!
//! The domain layer should NOT know about HTTP status codes or response formats.
//! Instead, errors provide semantic categorization (Authorization, NotFound, etc.)
//! and the driver layer maps these to HTTP concepts.

/// Metadata trait for domain and application errors
///
/// This trait provides a unified interface for extracting error information
/// without coupling the domain layer to HTTP or other infrastructure concerns.
///
/// # Example
///
/// ```ignore
/// impl ErrorMetadata for UserServiceError {
///     fn error_category(&self) -> ErrorCategory {
///         match self {
///             Self::Unauthorized => ErrorCategory::Authorization,
///             Self::NotFound { .. } => ErrorCategory::NotFound,
///         }
///     }
///
///     fn severity(&self) -> ErrorSeverity {
///         match self {
///             Self::Unauthorized => ErrorSeverity::Info,
///             Self::InternalError(_) => ErrorSeverity::Error,
///         }
///     }
///
///     fn user_message(&self) -> String {
///         match self {
///             Self::Unauthorized => "You can only access your own account".to_string(),
///         }
///     }
/// }
/// ```
pub trait ErrorMetadata: std::error::Error {
    /// Get the semantic category of this error
    ///
    /// Used to determine appropriate HTTP status codes and logging behavior
    fn error_category(&self) -> ErrorCategory;

    /// Get the severity level for logging and alerting
    fn severity(&self) -> ErrorSeverity;

    /// Get a safe, user-facing error message
    ///
    /// This message should never expose internal implementation details
    /// or sensitive information. It should be actionable for end users.
    fn user_message(&self) -> String;

    /// Get internal debug context (not exposed to users)
    ///
    /// This is used for internal logging and debugging only.
    /// Default implementation returns None.
    fn internal_context(&self) -> Option<String> {
        None
    }
}

/// Semantic error categories (infrastructure-independent)
///
/// These categories represent the semantic meaning of errors without
/// being tied to HTTP status codes or other protocol-specific concepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Authentication required or failed
    Authentication,

    /// Insufficient permissions for the requested operation
    Authorization,

    /// Input validation failed
    Validation,

    /// Requested resource does not exist
    NotFound,

    /// Resource already exists or operation conflicts with current state
    Conflict,

    /// Database operation failed
    Database,

    /// External service call failed
    ExternalService,

    /// Unexpected internal error
    Internal,
}

/// Error severity levels for logging and alerting
///
/// Severity determines how errors should be logged and whether they
/// require immediate attention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Expected user error (e.g., validation failure)
    ///
    /// These errors are normal part of application flow and typically
    /// don't need to be logged at error level.
    Info,

    /// Expected error that should be logged for monitoring
    ///
    /// Examples: rate limiting, business rule violations
    Warning,

    /// Unexpected error requiring investigation
    ///
    /// Examples: database failures, external service errors
    Error,

    /// Critical system error requiring immediate attention
    ///
    /// Examples: data corruption, security breaches
    Critical,
}

impl ErrorCategory {
    /// Get a human-readable name for this category
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Authentication => "Authentication",
            Self::Authorization => "Authorization",
            Self::Validation => "Validation",
            Self::NotFound => "NotFound",
            Self::Conflict => "Conflict",
            Self::Database => "Database",
            Self::ExternalService => "ExternalService",
            Self::Internal => "Internal",
        }
    }
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "Info"),
            Self::Warning => write!(f, "Warning"),
            Self::Error => write!(f, "Error"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}
