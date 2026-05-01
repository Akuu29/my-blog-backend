use crate::model::error::RepositoryError;
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait ITokenRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn fetch_jwks(&self) -> Result<HashMap<String, String>, RepositoryError>;
}
