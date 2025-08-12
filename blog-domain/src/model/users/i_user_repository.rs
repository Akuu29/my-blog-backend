use crate::model::{
    common::pagination::Pagination,
    users::user::{NewUser, UpdateUser, User},
};
use async_trait::async_trait;
use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};
use sqlx::types::Uuid;
use validator::Validate;

#[serde_as]
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UserFilter {
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub name_contains: Option<String>,
}

#[async_trait]
pub trait IUserRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, payload: NewUser) -> anyhow::Result<User>;
    async fn all(
        &self,
        user_filter: UserFilter,
        pagination: Pagination,
    ) -> anyhow::Result<Vec<User>>;
    async fn find(&self, user_id: Uuid) -> anyhow::Result<User>;
    async fn find_by_user_identity(
        &self,
        provider_name: &str,
        idp_sub: &str,
    ) -> anyhow::Result<User>;
    async fn update(&self, user_id: Uuid, payload: UpdateUser) -> anyhow::Result<User>;
    async fn delete(&self, user_id: Uuid) -> anyhow::Result<()>;
}
