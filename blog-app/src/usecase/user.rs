use crate::{
    model::user::{SigninUser, SignupUser, User},
    repository::user::UserRepository,
};

pub struct UserUseCase<T: UserRepository> {
    repository: T,
}

impl<T: UserRepository> UserUseCase<T> {
    pub fn new(repository: T) -> Self {
        Self { repository }
    }

    pub async fn signup(&self, payload: SignupUser) -> anyhow::Result<User> {
        self.repository.signup(payload).await
    }

    pub async fn signin(&self, payload: SigninUser) -> anyhow::Result<User> {
        self.repository.signin(payload).await
    }
}
