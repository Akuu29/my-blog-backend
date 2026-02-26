use crate::config::FirebaseConfig;
use async_trait::async_trait;
use blog_domain::model::tokens::i_token_repository::ITokenRepository;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TokenRepository {
    client: reqwest::Client,
    config: FirebaseConfig,
}

impl TokenRepository {
    pub fn new(client: reqwest::Client, config: FirebaseConfig) -> Self {
        Self { client, config }
    }
}

#[async_trait]
impl ITokenRepository for TokenRepository {
    async fn fetch_jwks(&self) -> anyhow::Result<HashMap<String, String>> {
        let jwks = self
            .client
            .get(&self.config.jwks_url)
            .send()
            .await?
            .json::<HashMap<String, String>>()
            .await?;

        Ok(jwks)
    }
}
