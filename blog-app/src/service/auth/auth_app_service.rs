use crate::model::auth::{
    auth::{SigninUser, SignupUser, UserCredentials},
    i_auth_repository::IAuthRepository,
};

pub struct AuthAppService<T: IAuthRepository> {
    repository: T,
}

impl<T: IAuthRepository> AuthAppService<T> {
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
