//! Error handling foundation for the domain layer

/// Semantic error categories (infrastructure-independent)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    Authentication,
    Authorization,
    Validation,
    NotFound,
    Conflict,
    Internal,
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Authentication => "Authentication",
            Self::Authorization => "Authorization",
            Self::Validation => "Validation",
            Self::NotFound => "NotFound",
            Self::Conflict => "Conflict",
            Self::Internal => "Internal",
        };
        write!(f, "{s}")
    }
}
