use thiserror::Error;

#[derive(Debug, Error)]
pub enum UsecaseError {
    #[error("ValidationFailed: [{0}]")]
    ValidationFailed(String),
    #[error("AuthenticationFailed: [{0}]")]
    AuthenticationFailed(String),
}
