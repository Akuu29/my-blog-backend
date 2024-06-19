use crate::repository::user::UserRepository;

pub struct UserUseCase<T: UserRepository> {
    repository: T,
}

impl<T: UserRepository> UserUseCase<T> {
    pub fn new(repository: T) -> Self {
        Self { repository }
    }

    pub async fn update(&self) -> anyhow::Result<()> {
        self.repository.update().await
    }

    pub async fn delete(&self) -> anyhow::Result<()> {
        self.repository.delete().await
    }
}
