use crate::model::{
    common::{item_count::ItemCount, pagination::Pagination},
    error::RepositoryError,
    users::user::{NewUser, UpdateUser, User},
};
use async_trait::async_trait;
use serde::Deserialize;
use sqlx::types::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UserFilter {
    #[validate(length(max = 100, message = "name_contains length must be 100 or less"))]
    pub name_contains: Option<String>,
}

#[async_trait]
pub trait IUserRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, payload: NewUser) -> Result<User, RepositoryError>;
    async fn all(
        &self,
        user_filter: UserFilter,
        pagination: Pagination,
    ) -> Result<(Vec<User>, ItemCount), RepositoryError>;
    async fn find(&self, user_id: Uuid) -> Result<User, RepositoryError>;
    async fn find_by_user_identity(
        &self,
        provider_name: &str,
        idp_sub: &str,
    ) -> Result<User, RepositoryError>;
    async fn update(&self, user_id: Uuid, payload: UpdateUser) -> Result<User, RepositoryError>;
    async fn delete(&self, user_id: Uuid) -> Result<(), RepositoryError>;
}
