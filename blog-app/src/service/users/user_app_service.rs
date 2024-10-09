use blog_domain::model::users::{
    i_user_repository::IUserRepository,
    user::{NewUser, UpdateUser, User},
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

    pub async fn find(&self, user_id: Uuid) -> anyhow::Result<User> {
        self.repository.find(user_id).await
    }

    // TODO Bad approach because it's not scalable
    pub async fn find_by_idp_sub(&self, idp_sub: &str) -> anyhow::Result<User> {
        self.repository.find_by_idp_sub(idp_sub).await
    }

    pub async fn update(&self, user_id: Uuid, payload: UpdateUser) -> anyhow::Result<User> {
        self.repository.update(user_id, payload).await
    }

    pub async fn delete(&self, user_id: Uuid) -> anyhow::Result<()> {
        self.repository.delete(user_id).await
    }
}
