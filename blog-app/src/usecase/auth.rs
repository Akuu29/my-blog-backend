use crate::{
    model::auth::{SigninUser, SignupUser, UserCredentials},
    repository::auth::AuthRepository,
};

pub struct AuthUseCase<T: AuthRepository> {
    repository: T,
}

impl<T: AuthRepository> AuthUseCase<T> {
    pub fn new(repository: T) -> Self {
        Self { repository }
    }

    pub async fn signup(&self, payload: SignupUser) -> anyhow::Result<UserCredentials> {
        self.repository.signup(payload).await
    }

    pub async fn signin(&self, payload: SigninUser) -> anyhow::Result<UserCredentials> {
        self.repository.signin(payload).await
    }

    pub async fn signout(&self, payload: SigninUser) -> anyhow::Result<UserCredentials> {
        todo!()
    }

    pub async fn refresh(&self, payload: SigninUser) -> anyhow::Result<UserCredentials> {
        todo!()
    }

    pub async fn reset_password(&self, payload: SigninUser) -> anyhow::Result<UserCredentials> {
        todo!()
    }
}
