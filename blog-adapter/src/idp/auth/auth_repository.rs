use async_trait::async_trait;
use blog_app::model::auth::{
    auth::{SigninUser, SignupUser, UserCredentials},
    i_auth_repository::IAuthRepository,
};
use std::env;

#[derive(Debug, Clone)]
pub struct AuthRepository {
    client: reqwest::Client,
    api_key: String,
}

impl AuthRepository {
    pub fn new(client: reqwest::Client, api_key: String) -> Self {
        Self { client, api_key }
    }
}

#[async_trait]
impl IAuthRepository for AuthRepository {
    async fn signup(&self, payload: SignupUser) -> anyhow::Result<UserCredentials> {
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

        let user_credentials = send_req.unwrap().json::<UserCredentials>().await?;

        Ok(user_credentials)
    }

    async fn signin(&self, payload: SigninUser) -> anyhow::Result<UserCredentials> {
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

        let user_credentials = send_req.unwrap().json::<UserCredentials>().await?;

        Ok(user_credentials)
    }

    async fn signout(&self, payload: SigninUser) -> anyhow::Result<()> {
        todo!()
    }

    async fn refresh(&self, payload: SigninUser) -> anyhow::Result<()> {
        todo!()
    }

    async fn reset_password(&self, payload: SigninUser) -> anyhow::Result<()> {
        todo!()
    }
}
