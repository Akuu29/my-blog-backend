use blog_domain::model::{
    common::pagination::Pagination,
    users::{
        i_user_repository::{IUserRepository, UserFilter},
        user::{NewUser, UpdateUser, User},
    },
};
use sqlx::types::Uuid;

pub struct UserAppService<T: IUserRepository> {
    repository: T,
}

impl<T: IUserRepository> UserAppService<T> {
    pub fn new(repository: T) -> Self {
        Self { repository }
    }

    pub async fn create(&self, payload: NewUser) -> anyhow::Result<User> {
        self.repository.create(payload).await
    }

    pub async fn all(
        &self,
        user_filter: UserFilter,
        pagination: Pagination,
    ) -> anyhow::Result<Vec<User>> {
        self.repository.all(user_filter, pagination).await
    }

    pub async fn find(&self, user_id: Uuid) -> anyhow::Result<User> {
        self.repository.find(user_id).await
    }

    pub async fn find_by_user_identity(
        &self,
        provider_name: &str,
        idp_sub: &str,
    ) -> anyhow::Result<User> {
        self.repository
            .find_by_user_identity(provider_name, idp_sub)
            .await
    }

    pub async fn update(&self, user_id: Uuid, payload: UpdateUser) -> anyhow::Result<User> {
        self.repository.update(user_id, payload).await
    }

    pub async fn delete(&self, user_id: Uuid) -> anyhow::Result<()> {
        self.repository.delete(user_id).await
    }
}
