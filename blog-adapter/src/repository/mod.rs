pub mod article;
pub mod auth;
pub mod comment;
pub mod token;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Unexpected Error: [{0}]")]
    Unexpected(String),
    #[error("NotFound")]
    NotFound,
}
