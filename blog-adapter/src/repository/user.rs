use crate::model::user::{SigninUser, SignupUser, User};
use async_trait::async_trait;
use std::env;

#[async_trait]
pub trait UserRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn signup(&self, payload: SignupUser) -> anyhow::Result<User>;
    async fn signin(&self, payload: SigninUser) -> anyhow::Result<User>;
}

#[derive(Debug, Clone)]
pub struct UserRepositoryForFirebase {
    client: reqwest::Client,
    api_key: String,
}

impl UserRepositoryForFirebase {
    pub fn new(client: reqwest::Client, api_key: String) -> Self {
        Self { client, api_key }
    }
}

#[async_trait]
impl UserRepository for UserRepositoryForFirebase {
    async fn signup(&self, payload: SignupUser) -> anyhow::Result<User> {
        dotenv::dotenv().ok();
        let firebase_signup_url =
            env::var("FIREBASE_SIGNUP_URL").expect("undefined FIREBASE_SIGNUP_URL");
        let url = format!("{}?key={}", firebase_signup_url, self.api_key);
        let send_req = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await;

        let user = send_req.unwrap().json::<User>().await?;

        Ok(user)
    }

    async fn signin(&self, payload: SigninUser) -> anyhow::Result<User> {
        dotenv::dotenv().ok();
        let firebase_find_url =
            env::var("FIREBASE_SIGNIN_URL").expect("undefined FIREBASE_FIND_URL");
        let url = format!("{}?key={}", firebase_find_url, self.api_key);
        let send_req = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await;

        let user = send_req.unwrap().json::<User>().await?;

        Ok(user)
    }
}
