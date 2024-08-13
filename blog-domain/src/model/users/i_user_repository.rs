use crate::model::users::user::{NewUser, UpdateUser, User};
use async_trait::async_trait;

#[async_trait]
pub trait IUserRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, payload: NewUser) -> anyhow::Result<User>;
    async fn find(&self, id: i32) -> anyhow::Result<User>;
    // TODO Bad approach because it's not scalable
    async fn find_by_idp_sub(&self, idp_sub: &str) -> anyhow::Result<User>;
    async fn update(&self, id: i32, payload: UpdateUser) -> anyhow::Result<User>;
    async fn delete(&self, id: i32) -> anyhow::Result<()>;
}
