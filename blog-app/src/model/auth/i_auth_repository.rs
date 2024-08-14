use crate::model::auth::auth::{SigninUser, SignupUser, UserCredentials};
use async_trait::async_trait;

#[async_trait]
pub trait IAuthRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn signup(&self, payload: SignupUser) -> anyhow::Result<UserCredentials>;
    async fn signin(&self, payload: SigninUser) -> anyhow::Result<UserCredentials>;
    async fn signout(&self, payload: SigninUser) -> anyhow::Result<()>;
    async fn refresh(&self, payload: SigninUser) -> anyhow::Result<()>;
    async fn reset_password(&self, payload: SigninUser) -> anyhow::Result<()>;
}
