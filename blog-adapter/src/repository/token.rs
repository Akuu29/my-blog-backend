use async_trait::async_trait;
use blog_app::repository::token::TokenRepository;
use std::{collections::HashMap, env};

#[derive(Debug, Clone)]
pub struct TokenRepositoryForFirebase {
    client: reqwest::Client,
}

impl TokenRepositoryForFirebase {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl TokenRepository for TokenRepositoryForFirebase {
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
