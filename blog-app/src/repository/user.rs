use crate::model::user::{SigninUser, SignupUser, User};
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn signup(&self, payload: SignupUser) -> anyhow::Result<User>;
    async fn signin(&self, payload: SigninUser) -> anyhow::Result<User>;
}
