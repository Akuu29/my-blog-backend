use blog_domain::model::users::{
    i_user_repository::IUserRepository,
    user::{NewUser, UpdateUser, User},
};

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

    pub async fn find(&self, id: i32) -> anyhow::Result<User> {
        self.repository.find(id).await
    }

    pub async fn update(&self, id: i32, payload: UpdateUser) -> anyhow::Result<User> {
        self.repository.update(id, payload).await
    }

    pub async fn delete(&self, id: i32) -> anyhow::Result<()> {
        self.repository.delete(id).await
    }
}
