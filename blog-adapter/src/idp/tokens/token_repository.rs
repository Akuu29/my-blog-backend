use async_trait::async_trait;
// use blog_app::repository::token::ITokenRepository;
use blog_domain::model::tokens::i_token_repository::ITokenRepository;
use std::{collections::HashMap, env};

#[derive(Debug, Clone)]
pub struct TokenRepository {
    client: reqwest::Client,
}

impl TokenRepository {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ITokenRepository for TokenRepository {
    async fn fetch_jwks(&self) -> anyhow::Result<std::collections::HashMap<String, String>> {
        let firebase_jwks_url = env::var("FIREBASE_JWKS_URL").expect("undefined FIREBASE_JWKS_URL");
        let jwks = self
            .client
            .get(firebase_jwks_url)
            .send()
            .await?
            .json::<HashMap<String, String>>()
            .await?;

        Ok(jwks)
    }
}
