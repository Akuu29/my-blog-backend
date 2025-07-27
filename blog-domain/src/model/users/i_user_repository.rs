use crate::model::users::user::{NewUser, UpdateUser, User};
use async_trait::async_trait;
use sqlx::types::Uuid;

#[async_trait]
pub trait IUserRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, payload: NewUser) -> anyhow::Result<User>;
    async fn find(&self, user_id: Uuid) -> anyhow::Result<User>;
    async fn find_by_user_identity(
        &self,
        provider_name: &str,
        idp_sub: &str,
    ) -> anyhow::Result<User>;
    async fn update(&self, user_id: Uuid, payload: UpdateUser) -> anyhow::Result<User>;
    async fn delete(&self, user_id: Uuid) -> anyhow::Result<()>;
}
