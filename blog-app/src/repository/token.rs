use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait TokenRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn fetch_jwks(&self) -> anyhow::Result<HashMap<String, String>>;
}
