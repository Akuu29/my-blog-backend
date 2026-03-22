use crate::service::error::UsecaseError;
use blog_domain::{
    model::{
        common::{item_count::ItemCount, pagination::Pagination},
        users::{
            i_user_repository::{IUserRepository, UserFilter},
            user::{NewUser, UpdateUser, User},
        },
    },
    service::users::UserService,
};
use sqlx::types::Uuid;

pub struct UserAppService<T: IUserRepository> {
    repository: T,
}

impl<T: IUserRepository> UserAppService<T> {
    pub fn new(repository: T) -> Self {
        Self { repository }
    }

    pub async fn create(&self, payload: NewUser) -> Result<User, UsecaseError> {
        Ok(self.repository.create(payload).await?)
    }

    pub async fn all(
        &self,
        user_filter: UserFilter,
        pagination: Pagination,
    ) -> Result<(Vec<User>, ItemCount), UsecaseError> {
        Ok(self.repository.all(user_filter, pagination).await?)
    }

    pub async fn find(&self, user_id: Uuid) -> Result<User, UsecaseError> {
        Ok(self.repository.find(user_id).await?)
    }

    pub async fn find_by_user_identity(
        &self,
        provider_name: &str,
        idp_sub: &str,
    ) -> Result<User, UsecaseError> {
        Ok(self
            .repository
            .find_by_user_identity(provider_name, idp_sub)
            .await?)
    }

    pub async fn update_with_auth(
        &self,
        user_id: Uuid,
        authenticated_user_id: Uuid,
        payload: UpdateUser,
    ) -> Result<User, UsecaseError> {
        // Verify that the user is acting on their own account
        UserService::verify_self(user_id, authenticated_user_id)?;

        Ok(self.repository.update(user_id, payload).await?)
    }

    pub async fn delete_with_auth(
        &self,
        user_id: Uuid,
        authenticated_user_id: Uuid,
    ) -> Result<(), UsecaseError> {
        // Verify that the user is acting on their own account
        UserService::verify_self(user_id, authenticated_user_id)?;

        self.repository.delete(user_id).await?;

        Ok(())
    }
}
